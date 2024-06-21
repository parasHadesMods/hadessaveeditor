mod gui;
mod luabins;
mod luastate;
mod read;
mod repl;
mod hadesfile;
mod write;

use anyhow::Result;
use clap::{arg, Command};
use hadesfile::HadesSave;
use rlua::Lua;
use std::path::{Path, PathBuf};
use std::fs;

fn cli() -> Command {
    Command::new("hadessaveeditor")
        .about("A save file editor for hades")
        .arg(arg!(file: [FILE] "The hades save file to open.").value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(-r --repl "Starts the command-line repl instead of the gui."))
        .arg(arg!(-s --script [SCRIPT] "Runs the script on the file before opening.").value_parser(clap::value_parser!(PathBuf)))
        .arg_required_else_help(true)
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    let path: &PathBuf = matches.get_one("file").expect("required");

    let lua = unsafe {
        Lua::new_with_debug()
    };

    let file = read_file(path)?;
    let savedata = hadesfile::read(&mut file.as_slice())?;
    let lua_state = match savedata.clone() {
        HadesSave::V16(data) => {
            luastate::initialize_v16(&lua)?;
            data.lua_state
        },
        HadesSave::V17(data) => {
            luastate::initialize_v17(&lua)?;
            data.lua_state
        }
    };

    luastate::load(&lua, &mut lua_state.as_slice())?;

    match matches.get_one::<PathBuf>("script") {
        Some(script_path) => {
            let file = read_file(script_path)?;
            lua.context(|ctx| -> Result<()> {
                let chunk = ctx.load(&file);
                chunk.exec().map_err(anyhow::Error::new)
            })?
        },
        _ => {}
    }

    if matches.get_flag("repl") {
        repl::repl(lua, savedata, path.to_owned())?;
    } else {
        gui::gui(lua, savedata, path.to_owned())?;
    }

    Ok(())
}

const BYTE_ORDER_MARK: &[u8] = "\u{feff}".as_bytes();
fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
  let file = fs::read(path)?;
  if file.starts_with(BYTE_ORDER_MARK) {
     Ok(file[3..].to_vec())
  } else {
     Ok(file.to_vec())
  }
}
