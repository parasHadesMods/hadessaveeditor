use crate::hadesfile;
use crate::luastate;

use anyhow::Result;
use hadesfile::HadesSaveV16;
use rlua::{Function, Lua, MultiValue};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
use std::path::PathBuf;

pub fn repl(lua: Lua, savedata: HadesSaveV16, path: PathBuf) -> Result<()> {
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
                println!("Saving {}", path.display());
                let mut savedata = savedata.clone();
                savedata.lua_state = luastate::save(&lua)?;
                let outfile = hadesfile::write(&savedata)?;
                fs::write(&path, outfile)?;
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