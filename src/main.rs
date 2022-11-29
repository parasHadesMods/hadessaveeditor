mod luabins;
mod luastate;
mod read;
mod hadesfile;
mod write;

use anyhow::Result;
use clap::{arg, Command};
use druid::im::Vector;
use druid::{AppLauncher, Color, Data, Env, Lens, Key, theme, Widget, WidgetExt, WindowDesc};
use druid::lens::{self, LensExt};
use druid::widget::{Flex, Label, List, Scroll};
use rlua::{Context, Function, Lua, MultiValue, Table, Value, FromLua};
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

    gui(&lua)?;

    savedata.lua_state = luastate::save(&lua)?;

    let outfile = hadesfile::write(&savedata)?;
    write_file(path, outfile)?;
    Ok(())
}

#[derive(Clone, Data, Lens)]
struct GuiState {
    dirty: bool,
    focus: Vector<String>,
    selected: Option<usize>,
    items: Vector<(String, String)>
}

const LABEL_TEXT_COLOR: Key<Color> = Key::new("paradigmsort.hadessaveeditor.label-text-color");

fn ui_builder() -> impl Widget<GuiState> {
    Scroll::new(
        List::new(|| {
            Flex::row()
                .with_flex_child(
                    Label::new(|(_selected, (_idx, item)): &(Option<usize>, (usize, (String, String))), _env: &_| item.0.clone())
                        .with_text_color(LABEL_TEXT_COLOR)
                        .on_click(|_ctx, (selected, (idx, item)), _env| {
                            if selected.unwrap_or(usize::MAX) == *idx {
                                selected.take();
                            } else {
                                selected.replace(*idx);
                            }
                        })
                        .expand_width(), 1.0)
                .with_flex_child(
                    Label::new(|(_selected, (_idx, item)): &(Option<usize>, (usize, (String, String))), _env: &_| item.1.clone())
                    .with_text_color(LABEL_TEXT_COLOR)
                    .expand_width(), 1.0)
                .env_scope(|env: &mut Env, (selected, (idx, _item)): &(Option<usize>, (usize, (String, String)))| {
                    let color = if selected.unwrap_or(usize::MAX) == *idx {
                        env.get(theme::SELECTION_TEXT_COLOR)
                    } else {
                        env.get(theme::LABEL_COLOR)
                    };
                    env.set(LABEL_TEXT_COLOR, color)
                })
        }).lens(lens::Identity.map(
            |data: &GuiState| {
                let indexed_items = Vector::from_iter(
                    data.items.iter().map(|item| item.clone()).enumerate());
                (data.selected, indexed_items)
            },
            |data: &mut GuiState, updated:(Option<usize>, Vector<(usize, (String, String))>)| {
                data.selected = updated.0
            }))
    ).vertical()
}

fn lua_is_saved_type(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Boolean(_) => true,
        Value::LightUserData(_) => false,
        Value::Integer(_) => true,
        Value::Number(_) => true,
        Value::String(_) => true,
        Value::Table(_) => true,
        Value::Function(_) => false,
        Value::Thread(_) => false,
        Value::UserData(_) => false,
        Value::Error(_) => false,
    }
}

fn lua_to_string<'a>(value: Value<'a>, lua_ctx: Context<'a>) -> Result<String> {
    let lua_string = match value {
        Value::Nil => "nil".to_owned(),
        Value::Boolean(boolean_value) => {
            if boolean_value { "true".to_owned() } else { "false".to_owned() }
        },
        Value::Integer(_) => {
            String::from_lua(value, lua_ctx)?
        },
        Value::Number(_) => {
            String::from_lua(value, lua_ctx)?
        },
        Value::String(_) => {
            String::from_lua(value, lua_ctx)?
        },
        Value::Table(_) => {
            "table".to_owned()
        },
        Value::Function(_) => {
            "function".to_owned()
        },
        Value::Thread(_) => todo!(),
        Value::UserData(_) => todo!(),
        Value::LightUserData(_) => todo!(),
        Value::Error(_) => todo!(),
    };
    Ok(lua_string)
}

fn gui(lua: &Lua) -> Result<()> {
    let mut gui_state = GuiState {
        dirty: false,
        selected: None,
        focus: Vector::new(),
        items: Vector::new()
    };
    lua.context(|lua_ctx| -> Result<()> {
        let save_ignores: Table = lua_ctx.globals().get("SaveIgnores")?;

        for pair in lua_ctx.globals().pairs::<Value, Value>() {
            let (key, value) = pair?;
            if lua_is_saved_type(&value) && !save_ignores.get(key.clone())? {
                gui_state.items.push_back((lua_to_string(key, lua_ctx)?, lua_to_string(value, lua_ctx)?));
            }
        }
        Ok(())
    })?;
    AppLauncher::with_window(WindowDesc::new(ui_builder))
        .launch(gui_state)?;
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
