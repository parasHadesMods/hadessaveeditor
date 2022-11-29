mod luabins;
mod luastate;
mod read;
mod hadesfile;
mod write;

use anyhow::Result;
use clap::{arg, Command};
use rlua::{Function, Lua, MultiValue};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
use std::path::{Path, PathBuf};

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

    repl(&lua)?;

    savedata.lua_state = luastate::save(&lua)?;

    let outfile = hadesfile::write(&savedata)?;
    write_file(path, outfile)?;
    Ok(())
}

fn repl(lua: &Lua) -> Result<()> {
    let mut editor = Editor::<()>::new()?;
    loop {
        let readline = editor.readline(">> ");

        //  TODO validation with rustyline::validte::Validator
        //  TODO hinting ?

        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_str());
                lua.context(|lua_ctx| -> Result<()> {
                    let result: MultiValue = lua_ctx.load(&line).eval()?;
                    let print: Function = lua_ctx.globals().get("print")?;
                    print.call(result)?;
                    Ok(())
                })?
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break
            },
            Err(err) => {
                println!("Unknown error: {:?}", err);
                break
            }
        }
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

fn write_file<P: AsRef<Path>>(path: P, data: Vec<u8>) -> Result<()> {
    fs::write(path, data).map_err(anyhow::Error::new)
}
