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
use rlua::{Context, Lua, Value as LuaValue};
use std::path::{Path, PathBuf};
use std::fs;
use serde_json::{Value, json};

fn cli() -> Command {
    Command::new("hadessaveeditor")
        .about("A save file editor for hades")
        .arg(arg!(file: [FILE] "The hades save file to open.").value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(-r --repl "Starts the command-line repl instead of the gui."))
        .arg(arg!(--json "Dumps the save as json instead of starting the gui."))
        .arg_required_else_help(true)
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    let path: &PathBuf = matches.get_one("file").expect("required");

    let lua = unsafe {
        Lua::new_with_debug()
    };

    let file = read_file(path)?;
    let savedata: HadesSave = hadesfile::read(&mut file.as_slice())?;
    let lua_state = match savedata.clone() {
        HadesSave::V16(data) => data.lua_state,
        HadesSave::V17(data) => data.lua_state
    };

    if matches.get_flag("json") {
        let mut value = dump_json(savedata)?;
        value["lua_state"] = lua.context(|lua_ctx| -> Result<Value> {
            let loaded = luabins::load(&mut lua_state.as_slice(), lua_ctx)?;
            dump_lua_json_vec(lua_ctx, loaded)
        })?;
        println!("{}", value);
    } else {
        luastate::initialize(&lua)?;
        luastate::load(&lua, &mut lua_state.as_slice())?;    

        if matches.get_flag("repl") {
            repl::repl(lua, savedata, path.to_owned())?;
        } else {
            gui::gui(lua, savedata, path.to_owned())?;
        }
    }

    Ok(())
}

fn dump_lua_json_vec(context: Context, values: Vec<LuaValue>) -> Result<Value> {
    let mut table = json!({});
    for (i, value) in values.iter().enumerate() {
        table[format!("[{}]", i)] = dump_lua_json(context, value)?
    }
    Ok(table)
}

fn dump_lua_json(context: Context, value: &LuaValue) -> Result<Value> {
    match value {
        LuaValue::Boolean(b) => Ok(json!(b)),
        LuaValue::Nil => Ok(json!(null)),
        LuaValue::Integer(i) => Ok(json!(i)),
        LuaValue::Number(n) => Ok(json!(n)),
        LuaValue::String(s) => Ok(json!(s.to_str()?)),
        LuaValue::Table(t) => {
            let mut table = json!({});
            t.clone().pairs::<LuaValue, LuaValue>().for_each(|pair| {
                let pair = pair.unwrap();
                let key = pair.0;
                let value = pair.1;

                match key {
                    LuaValue::String(s) => {
                        table[s.to_str().unwrap()] = dump_lua_json(context, &value).unwrap()
                    },
                    LuaValue::Integer(i) => {
                        table[format!("[{}]", i)] = dump_lua_json(context, &value).unwrap()
                    },
                    _ => {}
                }
            });
            Ok(table)
        },
        LuaValue::Function(_) => todo!(),
        LuaValue::Thread(_) => todo!(),
        LuaValue::UserData(_) => todo!(),
        LuaValue::LightUserData(_) => todo!(),
        LuaValue::Error(_) => todo!(),
    }
}

fn dump_json(save: HadesSave) -> Result<Value> {
    let value = match save {
        HadesSave::V16(data) => 
            json!({
                "timestamp": data.timestamp,
                "location": data.location,
                "runs": data.runs,
                "active_meta_points": data.active_meta_points,
                "active_shrine_points": data.active_shrine_points,
                "god_mode_enabled": data.god_mode_enabled,
                "hell_mode_enabled": data.hell_mode_enabled,
                "lua_keys": data.lua_keys,
                "current_map_name": data.current_map_name,
                "start_next_map": data.start_next_map,
            }),
        HadesSave::V17(data) => 
            json!({
                "timestamp": data.timestamp,
                "location": data.location,
                "padding1": data.padding1,
                "runs": data.runs,
                "god_mode_enabled": data.god_mode_enabled,
                "hell_mode_enabled": data.hell_mode_enabled,
                "lua_keys": data.lua_keys,
                "current_map_name": data.current_map_name,
                "start_next_map": data.start_next_map,
            }),
    };

    Ok(value)
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
