mod luabins;
mod read;
mod save;

use anyhow::Result;
use clap::{arg, Command};
use rlua::{Function, Lua, MultiValue};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use save::UncompressedSize;
use std::fs;
use std::path::{Path, PathBuf};
use lz4;

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
    let savedata = save::read(&mut file.as_slice())?;
    let lua_state = lz4::block::decompress(
        &savedata.lua_state_lz4.as_slice(), 
        Some(save::HadesSaveV16::UNCOMPRESSED_SIZE))?;


    lua.context(|lua_ctx| -> Result<()> {
        let save_data = luabins::load(&mut lua_state.as_slice(), lua_ctx)?;
        lua_ctx.globals().set("HadesSaveEditorSaveData", save_data)?;
        // put save file data into globals
        lua_ctx.load(r#"
            SaveIgnores = {}
            for _,savedValues in pairs(HadesSaveEditorSaveData) do
                for key, value in pairs(savedValues) do
                if not SaveIgnores[key] then
                    _G[key] = value
                end
                end
            end
            HadesSaveEditorSaveData = nil
        "#).exec().map_err(anyhow::Error::new)
    })?;

    repl(lua)
}

fn repl(lua: Lua) -> Result<()> {
    let mut editor = Editor::<()>::new()?;
    loop {
        let readline = editor.readline(">> ");

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
