use crate::hadesfile;
use crate::luastate;

use anyhow::{bail, Result};
use druid::im::Vector;
use druid::{AppLauncher, Color, Data, Env, Lens, Key, Size, theme, Widget, WidgetExt, WindowDesc};
use druid::lens::{self, LensExt};
use druid::widget::{Button, Flex, Label, List, Scroll, TextBox, Either};
use hadesfile::HadesSaveV16;
use rlua::{Context, Lua, Table, Value, FromLua};
use std::fs;
use std::rc::Rc;
use std::path::PathBuf;


#[derive(Clone, Data, Lens)]
struct GuiState {
    lua: Rc<Lua>,
    #[data(ignore)]
    path: PathBuf,
    #[data(ignore)]
    savedata: HadesSaveV16,
    dirty: bool,
    columns: Vector<Column>,
    lua_path_pointed_by_columns: Vector<TableKey>,
    value_pointed_by_columns: Option<String>,
    value_edit_box: String
}

impl GuiState {
    fn run_command(self: &mut GuiState, command: &str) -> Result<()> {
        self.lua.context(|lua_ctx| -> Result<()> {
            lua_ctx.load(command).exec().map_err(anyhow::Error::new)
        })?;
        self.dirty = true;
        self.sync()
    }
    fn sync(self: &mut GuiState) -> Result<()> {
        self.columns = Vector::new();
        self.lua.context(|lua_ctx| -> Result<()> {
            let save_ignores: Table = lua_ctx.globals().get("SaveIgnores")?;

            let mut lua_path: Vector<TableKey> = Vector::new();
            let mut idx = 0;
            let mut lua_path_iter = self.lua_path_pointed_by_columns.iter();
            loop {
                let lua_value = lua_get_path(lua_ctx, lua_path.clone())?;
                match lua_value {
                    Value::Table(table_value) => {
                        self.columns.push_back(Column { selected: None, items: Vector::new() });
                        for pair in table_value.pairs::<Value, Value>() {
                            let (key, value) = pair?;
                            if lua_is_saved_type(&value) && (idx != 1 ||!save_ignores.get(key.clone())?) {
                                self.columns[idx].items.push_back(lua_to_table_key(key, lua_ctx)?);
                            }
                        }
                        self.columns[idx].items.sort();
                    },
                    _ => {
                        self.value_pointed_by_columns = Some(lua_to_string(lua_value, lua_ctx)?);
                    }
                }
                match lua_path_iter.next() {
                    Some(segment) => {
                        lua_path.push_back(segment.to_owned());
                        idx += 1;
                    },
                    None => {
                        break;
                    },
                }
            };
            Ok(())
        })
    }
}

#[derive(Clone, Data, Lens)]
struct Column {
    selected: Option<usize>,
    items: Vector<TableKey>
}

#[derive(Clone, Data, PartialEq, PartialOrd, Eq, Ord)]
enum TableKey {
    StringKey(String),
    NumberKey(i64)
} 

const LABEL_TEXT_COLOR: Key<Color> = Key::new("paradigmsort.hadessaveeditor.label-text-color");

fn ui_builder() -> impl Widget<GuiState> {
    let columns = List::new(|| {
        Scroll::new(
        List::new( || {
            Label::new(|(_selected, (_idx, name)): &(Option<usize>, (usize, TableKey)), _env: &_| {
                match name {
                    TableKey::StringKey(string_key) => string_key.clone(),
                    TableKey::NumberKey(number_key) => format!("[{}]", number_key),
                }
            })
                .with_text_color(LABEL_TEXT_COLOR)
                .env_scope(|env: &mut Env, (selected, (idx, _item)): &(Option<usize>, (usize, TableKey))| {
                    let color = if selected.unwrap_or(usize::MAX) == *idx {
                        env.get(theme::SELECTION_TEXT_COLOR)
                    } else {
                        env.get(theme::LABEL_COLOR)
                    };
                    env.set(LABEL_TEXT_COLOR, color)
                })
                .on_click(|_ctx, (selected, (idx, _item)), _env| {
                    if selected.unwrap_or(usize::MAX) == *idx {
                        selected.take();
                    } else {
                        selected.replace(*idx);
                    }
                })
        })).vertical().lens(lens::Identity.map(
            |data: &Column| { (
                data.selected,
                Vector::from_iter(data.items.iter().map(|item| item.clone()).enumerate()))
            },
            |data: &mut Column, updated: (Option<usize>, Vector<(usize, TableKey)>) | {
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
                for (index, (old, new)) in data.columns.iter().zip(updated.iter()).enumerate() {
                    if old.selected != new.selected {
                        changed_index = index;
                        data.lua_path_pointed_by_columns = data.lua_path_pointed_by_columns.take(changed_index);
                        for selected_idx in new.selected {
                            data.lua_path_pointed_by_columns.push_back(new.items[selected_idx].clone())
                        }
                        break;
                    }
                }
                if changed_index != usize:: MAX {
                    data.columns = updated.take(changed_index + 1);
                    if data.columns[changed_index].selected.is_some() {
                        data.lua.context(|lua_ctx| -> Result<()> {
                            let lua_value_at_path = lua_get_path(lua_ctx, data.lua_path_pointed_by_columns.clone())?;
                            match lua_value_at_path {
                                Value::Table(table_value) => {
                                    data.columns.push_back(Column {
                                        selected: None,
                                        items: Vector::new()
                                    });
                                    for pair in table_value.pairs::<Value, Value>() {
                                        let (key, value) = pair?;
                                        if lua_is_saved_type(&value) {
                                            data
                                                .columns[changed_index + 1]
                                                .items
                                                .push_back(lua_to_table_key(key, lua_ctx)?);
                                        }
                                    }
                                    data.columns[changed_index + 1].items.sort();
                                    data.value_pointed_by_columns = None;
                                    data.value_edit_box.clear();
                                    Ok(())
                                },
                                _ => {
                                    let lua_string = lua_to_string(lua_value_at_path, lua_ctx)?;
                                    data.value_pointed_by_columns = Some(lua_string.clone());
                                    data.value_edit_box = lua_string;
                                    Ok(())
                                },
                            }
                        }).unwrap() // TODO!
                    }
                }
            }
        }
    ));

    let file_row =
        Flex::row()
            .with_flex_child(Either::new(
                |dirty: &bool, _env: &_| dirty.clone(),
                Label::new("You have unsaved changes."),
                Label::new("All changes have been saved!")
            ).lens(GuiState::dirty), 1.)
            .with_child(Button::new("Save").on_click(|_ctx, state: &mut GuiState, _env| {
                if state.dirty {
                    state.savedata.lua_state = luastate::save(state.lua.as_ref()).unwrap(); // TODO
                    let outfile = hadesfile::write(&state.savedata).unwrap(); // TODO
                    fs::write(&state.path, outfile).unwrap(); // TODO
                    state.dirty = false;
                }
            }))
            .padding(5.);

    let path_row =
        Flex::row()
            .with_child(Label::new("Focus"))
            .with_spacer(8.)
            .with_flex_child(
                Label::new(| lua_path: &Vector<TableKey>, _env: &_ | {
                    lua_path_as_string(lua_path)
                })
                .expand_width()
                .lens(GuiState::lua_path_pointed_by_columns), 1.)
            .padding(5.);

    let value_row  =
        Flex::row()
            .with_child(Label::new("Value"))
            .with_spacer(8.)
            .with_flex_child(TextBox::new().lens(GuiState::value_edit_box).expand_width(), 1.)
            .with_child(Button::new("Apply")
                .on_click(|_ctx, state: &mut GuiState, _env| {
                    let mut command: String = lua_path_as_string(&state.lua_path_pointed_by_columns);
                    command.push_str(" = ");
                    command.push_str(&state.value_edit_box);
                    state.run_command(&command).unwrap(); // TODO
                }))
            .padding(5.);

    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_child(file_row)
        .with_child(path_row)
        .with_child(value_row)
        .with_flex_child(columns, 1.0)
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

fn lua_get_path<'a>(lua_ctx: Context<'a>, lua_path: Vector<TableKey>) -> Result<Value<'a>> {
    let mut current_value: Value = Value::Table(lua_ctx.globals());
    for segment in lua_path {
        match current_value {
            Value::Table(table_value) => {
                match segment {
                    TableKey::NumberKey(i) => {
                        current_value = table_value.get(i)?;
                    },
                    TableKey::StringKey(s) => {
                        current_value = table_value.get(s)?;
                    }
                }
            },
            _ => bail!("not a table! {:?}", current_value)
        }
    }
    Ok(current_value)
}

fn lua_path_as_string(lua_path: &Vector<TableKey>) -> String {
    let mut path_string: String = "".to_owned();
    for segment in lua_path {
        match segment {
            TableKey::NumberKey(i) => {
                path_string.push_str(&format!("[{}]", i))
            },
            TableKey::StringKey(s) => {
                if path_string != "" {
                    path_string.push_str(".");
                }
                path_string.push_str(&s);
            }
        }
    }
    path_string
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

fn lua_to_table_key<'a>(value: Value<'a>, lua_ctx: Context<'a>) -> Result<TableKey> {
    let table_key = match value {
        Value::Integer(i) => TableKey::NumberKey(i),
        Value::String(_) => TableKey::StringKey(String::from_lua(value, lua_ctx)?),
        _ => todo!()
    };
    Ok(table_key)
}

pub fn gui(lua: Lua, savedata: HadesSaveV16, path: PathBuf) -> Result<()> {
    let mut gui_state = GuiState {
        lua: Rc::new(lua),
        path: path,
        savedata: savedata,
        dirty: false,
        columns: Vector::new(),
        lua_path_pointed_by_columns: Vector::new(),
        value_pointed_by_columns: None,
        value_edit_box: String::new()
    };
    gui_state.sync()?;

    let main_window = WindowDesc::new(ui_builder)
        .title(|state: &GuiState, _env: &_| format!(
            "Hades Save Editor - {}{}",
            state.path.display(),
            if state.dirty {"*"} else {""}))
        .window_size(Size::new(900.0, 800.0));

    AppLauncher::with_window(main_window)
        .launch(gui_state)?;
    Ok(())
}