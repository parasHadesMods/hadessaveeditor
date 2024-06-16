use crate::read;
use crate::write;

use anyhow::{anyhow, Context, Result};
use rlua::Lua;
use std::convert::TryInto;
use rlua::Value;

const LUABINS_CNIL: u8 = 0x2D;
const LUABINS_CFALSE: u8 = 0x30;
const LUABINS_CTRUE: u8 = 0x31;
const LUABINS_CNUMBER: u8 = 0x4E;
const LUABINS_CSTRING: u8 = 0x53 ;
const LUABINS_CTABLE: u8 = 0x54;

fn load_number<'lua>(loadstate: &mut &[u8]) -> Result<rlua::Value<'lua>> {
  let float = read::f64(loadstate)?;
  //println!("number {}", float);
  if float.fract() == 0.0 {
    Ok(Value::Integer(float.trunc() as i64))
  } else {
    Ok(Value::Number(float))
  }
}

fn load_string<'lua>(loadstate: &mut &[u8], context: rlua::Context<'lua>) -> Result<rlua::String<'lua>> {
  let len = read::u32(loadstate).context("string size")?;
  let str_bytes = read::bytes(loadstate, len.try_into().unwrap()).context("string")?;
  //println!("string {} {:?}", len, std::str::from_utf8(str_bytes));
  context.create_string(str_bytes).map_err(anyhow::Error::new)
}

fn load_table<'lua>(loadstate: &mut &[u8], context: rlua::Context<'lua>) -> Result<rlua::Table<'lua>> {
  let array_size = read::i32(loadstate).context("array_size")?;
  let hash_size = read::i32(loadstate).context("hash_size")?;
  let total_size = array_size + hash_size;
  let table: rlua::Table<'lua> = context.create_table().context("create_table")?;

  //println!("table arr {} hash {}", array_size, hash_size);
  for _ in 0..total_size {
    let key = load_value(loadstate, context).context("key")?;
    let value = load_value(loadstate, context).context("value")?;
    table.set(key, value).context("table.set")?;
  }
  Ok(table)
}

fn load_value<'a>(loadstate: &mut &[u8], context: rlua::Context<'a>) -> Result<Value<'a>> {
  let tbyte = read::byte(loadstate).context("type")?;
  //println!("type {}", tbyte);
  match tbyte {
    LUABINS_CNIL => Ok(Value::Nil),
    LUABINS_CFALSE => Ok(Value::Boolean(false)),   
    LUABINS_CTRUE => Ok(Value::Boolean(true)),   
    LUABINS_CNUMBER => Ok(load_number(loadstate).context("cnumber")?),
    LUABINS_CSTRING => Ok(Value::String(load_string(loadstate, context).context("cstring")?)),
    LUABINS_CTABLE => Ok(Value::Table(load_table(loadstate, context).context("ctable")?)),
    _ => Err(anyhow!("unknown type {}", tbyte))
  }
}

pub fn load<'lua>(loadstate: &mut &[u8], context: rlua::Context<'lua>) -> Result<Vec<Value<'lua>>> {
    println!("load size {} [{} {}]", loadstate.len(), loadstate.first().unwrap(), loadstate.last().unwrap());
    let num_items = read::byte(loadstate).context("num_items")?;
    //println!("load {}", num_items);
    let mut vec = Vec::new();
    for _ in 0..num_items {
        let value = load_value(loadstate, context).context("load")?;
        vec.push(value);
    }
    println!("load remaining {} all0 {}",
      loadstate.len(),
      loadstate.iter().all(|x| x == &b'\0'));
    Ok(vec)
}

pub fn size(loadstate: &[u8]) -> Result<usize> {
  Lua::new().context(|context| -> Result<usize> {
    let ugh = loadstate.to_owned();
    let copy = &mut ugh.as_slice();
    let num_items = read::byte(copy).context("num_items")?;
    for _ in 0..num_items {
        load_value(copy, context).context("load")?;
    }

    let size = loadstate.len() - copy.len();
    println!("init {} rem {} size {}",
      loadstate.len(),
      copy.len(),
      size);

    Ok(size)
  })
}

fn save_string(savestate: &mut Vec<u8>, string: rlua::String) {
  let mut str_bytes = string.as_bytes().to_owned();
  write::u32(savestate, str_bytes.len() as u32);
  write::bytes(savestate, &mut str_bytes)
}

fn save_table(savestate: &mut Vec<u8>, table: rlua::Table) -> Result<()> {
  // If there are holes punched in the "array section", len
  // will overreport the size. Luabins handles this by limiting
  // array_size to the total_size.
  let total_size = table.clone().pairs::<Value, Value>().count() as i32;
  let array_size = std::cmp::min(total_size, table.len()? as i32);
  write::i32(savestate, array_size);
  let hash_size = std::cmp::max(0, total_size - array_size);
  write::i32(savestate, hash_size);
  for pair in table.pairs::<Value, Value>() {
    let (key, value) = pair?;
    save_value(savestate, key)?;
    save_value(savestate, value)?;
  }
  Ok(())
}

fn save_value(savestate: &mut Vec<u8>, value: Value) -> Result<()> {
  match value {
    Value::Nil => {
      write::byte(savestate, LUABINS_CNIL);
      Ok(())
    },
    Value::Boolean(boolean_value) => {
      if boolean_value {
        write::byte(savestate, LUABINS_CTRUE)
      } else {
        write::byte(savestate, LUABINS_CFALSE)
      }
      Ok(())
    },
    Value::Integer(integer_value) => {
      write::byte(savestate, LUABINS_CNUMBER);
      write::f64(savestate, integer_value as f64);
      Ok(())
    },
    Value::Number(number_value) => {
      write::byte(savestate, LUABINS_CNUMBER);
      write::f64(savestate, number_value);
      Ok(())
    },
    Value::String(string_value) => {
      write::byte(savestate, LUABINS_CSTRING);
      save_string(savestate, string_value);
      Ok(())
    },
    Value::Table(table_value) => {
      write::byte(savestate, LUABINS_CTABLE);
      save_table(savestate, table_value)
    },
    // Skip the esoteric types; they don't appear in Hades saves.
    Value::Error(_) => Ok(()),
    Value::Function(_) => Ok(()),
    Value::LightUserData(_) => Ok(()),
    Value::UserData(_) => Ok(()),
    Value::Thread(_) => Ok(()),
  }
}

pub fn save<'lua>(savestate: &mut Vec<u8>, values: Vec<Value>) -> Result<()> {
  write::byte(savestate, values.len() as u8);
  for value in values.into_iter() {
    save_value(savestate, value)?;
  }
  Ok(())
}