mod gui;
mod luabins;
mod luastate;
mod read;
mod repl;
mod hadesfile;
mod write;

use anyhow::Result;
use clap::{arg, Command};
use rlua::Lua;
use std::path::{Path, PathBuf};
use std::fs;

fn cli() -> Command {
    Command::new("hadessaveeditor")
        .about("A save file editor for hades")
        .arg(arg!(file: [FILE]).value_parser(clap::value_parser!(PathBuf)))
        .arg_required_else_help(true)
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    let path: &PathBuf = matches.get_one("file").expect("required");

    let lua = unsafe {
        Lua::new_with_debug()
    };

    let file = read_file(path)?;
    let mut savedata = hadesfile::read(&mut file.as_slice())?;

    luastate::initialize(&lua)?;
    luastate::load(&lua, &mut savedata.lua_state.as_slice())?;

    gui::gui(lua)?;

    // savedata.lua_state = luastate::save(&lua)?;

    // let outfile = hadesfile::write(&savedata)?;
    // write_file(path, outfile)?;
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

fn write_file<P: AsRef<Path>>(path: P, data: Vec<u8>) -> Result<()> {
    fs::write(path, data).map_err(anyhow::Error::new)
}
