use anyhow::{bail, Result};
use druid::im::Vector;
use druid::{AppLauncher, Color, Data, Env, Lens, Key, theme, Widget, WidgetExt, WindowDesc};
use druid::lens::{self, LensExt};
use druid::widget::{Label, List, Scroll};
use rlua::{Context, Lua, Table, Value, FromLua};
use std::rc::Rc;

#[derive(Clone, Data, Lens)]
struct GuiState {
    lua: Rc<Lua>,
    generation: u64,
    columns: Vector<Column>,
    value_pointed_by_columns: Option<String>
}

#[derive(Clone, Data, Lens)]
struct Column {
    selected: Option<usize>,
    items: Vector<String>
}

const LABEL_TEXT_COLOR: Key<Color> = Key::new("paradigmsort.hadessaveeditor.label-text-color");

fn ui_builder() -> impl Widget<GuiState> {
    List::new(|| {
        Scroll::new(
        List::new( || {
            Label::new(|(_selected, (_idx, name)): &(Option<usize>, (usize, String)), _env: &_| name.clone())
                .with_text_color(LABEL_TEXT_COLOR)
                .env_scope(|env: &mut Env, (selected, (idx, _item)): &(Option<usize>, (usize, String))| {
                    let color = if selected.unwrap_or(usize::MAX) == *idx {
                        env.get(theme::SELECTION_TEXT_COLOR)
                    } else {
                        env.get(theme::LABEL_COLOR)
                    };
                    env.set(LABEL_TEXT_COLOR, color)
                })
                .on_click(|_ctx, (selected, (idx, item)), _env| {
                    if selected.unwrap_or(usize::MAX) == *idx {
                        selected.take();
                    } else {
                        selected.replace(*idx);
                    }
                })
        })).lens(lens::Identity.map(
            |data: &Column| { (
                data.selected,
                Vector::from_iter(data.items.iter().map(|item| item.clone()).enumerate()))
            },
            |data: &mut Column, updated: (Option<usize>, Vector<(usize, String)>) | {
                data.selected = updated.0
            }
        ))
    })
    .horizontal()
    .lens(lens::Identity.map(
        |data: &GuiState | { data.columns.clone() },
        |data: &mut GuiState, updated: Vector<Column> | {
            if ! data.columns.same(&updated) {
                let mut changed_index = usize::MAX;
                let mut lua_path = Vector::new(); 
                for (index, (old, new)) in data.columns.iter().zip(updated.iter()).enumerate() {
                    for selected_idx in new.selected {
                        lua_path.push_back(old.items[selected_idx].clone())
                    }
                    if old.selected != new.selected {
                        changed_index = index;
                        println!("path {:#?}", lua_path);
                        println!("changed {}", changed_index);
                        break;
                    }
                }
                if (changed_index != usize:: MAX) {
                    data.columns = updated.take(changed_index + 1);
                    data.lua.context(|lua_ctx| -> Result<()> {
                        let lua_value_at_path = lua_get_path(lua_ctx, lua_path)?;
                        match lua_value_at_path {
                            Value::Table(table_value) => {
                                data.columns.push_back(Column {
                                    selected: None,
                                    items: Vector::new()
                                });
                                for pair in lua_ctx.globals().pairs::<Value, Value>() {
                                    let (key, value) = pair?;
                                    if lua_is_saved_type(&value) {
                                        data
                                            .columns[changed_index + 1]
                                            .items
                                            .push_back(lua_to_string(key, lua_ctx)?);
                                    }
                                }
                                data.value_pointed_by_columns = None;
                                Ok(())
                            },
                            _ => {
                                data.value_pointed_by_columns = Some(lua_to_string(lua_value_at_path, lua_ctx)?);
                                Ok(())
                            },
                        }
                        
                    }).unwrap() // TODO!
                }
            }
        }
    ))
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

fn lua_get_path(lua_ctx: Context, lua_path: Vector<String>) -> Result<Value> {
    let mut current_value: Value = Value::Table(lua_ctx.globals());
    for segment in lua_path {
        match current_value {
            Value::Table(table_value) => {
                current_value = table_value.get(segment)?;
            },
            _ => bail!("not a table! {:?}", current_value)
        }
    }
    Ok(current_value)
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

pub fn gui(lua: Lua) -> Result<()> {
    let mut gui_state = GuiState {
        lua: Rc::new(lua),
        generation: 0,
        columns: Vector::new(),
        value_pointed_by_columns: None
    };
    gui_state.columns.push_back(Column {
        selected: None,
        items: Vector::new()
    });

    gui_state.lua.context(|lua_ctx| -> Result<()> {
        let save_ignores: Table = lua_ctx.globals().get("SaveIgnores")?;

        for pair in lua_ctx.globals().pairs::<Value, Value>() {
            let (key, value) = pair?;
            if lua_is_saved_type(&value) && !save_ignores.get(key.clone())? {
                gui_state.columns[0].items.push_back(lua_to_string(key, lua_ctx)?);
            }
        }
        Ok(())
    })?;

    AppLauncher::with_window(WindowDesc::new(ui_builder))
        .launch(gui_state)?;
    Ok(())
}