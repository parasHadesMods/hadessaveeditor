use anyhow::Result;
use rlua::{Function, Lua, MultiValue};
use rustyline::error::ReadlineError;
use rustyline::Editor;

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