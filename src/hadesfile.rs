use crate::luabins;
use crate::read;
use crate::write;

use adler32::adler32;
use anyhow::{bail, Context, Result};
use lz4;
use std::convert::TryInto;

pub trait UncompressedSize {
  const UNCOMPRESSED_SIZE: i32;
}

#[derive(Clone)]
pub enum HadesSave {
  V16(HadesSaveV16),
  V17(HadesSaveV17)
}

#[derive(Clone)]
pub struct HadesSaveV16 {
  pub timestamp: u64,
  pub location: String,
  pub runs: u32,
  pub active_meta_points: u32,
  pub active_shrine_points: u32,
  pub god_mode_enabled: bool,
  pub hell_mode_enabled: bool,
  pub lua_keys: Vec<String>,
  pub current_map_name: String,
  pub start_next_map: String,
  pub lua_state: Vec<u8>
}

impl UncompressedSize for HadesSaveV16 {
  const UNCOMPRESSED_SIZE: i32 = 9388032;
}

#[derive(Clone)]
pub struct HadesSaveV17 { // Hades 2
  pub timestamp: u64,
  pub location: String,
  pub padding1: Vec<u8>, // 12 bytes
  pub runs: u32,
  pub god_mode_enabled: bool,
  pub hell_mode_enabled: bool,
  pub lua_keys: Vec<String>,
  pub current_map_name: String,
  pub start_next_map: String,
  pub lua_state: Vec<u8>
}


fn read_string(loadstate: &mut &[u8]) -> Result<String> {
  let size = read::u32(loadstate).context("size")?;
  let str_bytes = read::bytes(loadstate, size.try_into().unwrap()).context("bytes")?;
  String::from_utf8(str_bytes.to_vec()).context("utf8")
}

pub fn read(loadstate: &mut &[u8]) -> Result<HadesSave> {
  let signature = read::bytes(loadstate, 4).context("signature")?;
  if signature != "SGB1".as_bytes() {
    bail!("Not a Hades save file");
  }
  let _checksum = read::bytes(loadstate, 4).context("checksum")?;
  let version = read::u32(loadstate).context("version")?;
  if version == 16 {
    return Ok(HadesSave::V16(read_v16(loadstate)?));
  } else if version == 17 {
    return Ok(HadesSave::V17(read_v17(loadstate)?));
  } else {
    bail!("unknown version {}", version);
  };
}

pub fn read_v16(loadstate: &mut &[u8]) -> Result<HadesSaveV16> {
  let timestamp = read::u64(loadstate).context("timestamp")?;
  let location = read_string(loadstate).context("location")?;
  let runs = read::u32(loadstate).context("runs")?;
  let active_meta_points = read::u32(loadstate).context("active_meta_points")?;
  let active_shrine_points = read::u32(loadstate).context("active_shrine_points")?;
  let god_mode_enabled = read::byte(loadstate).context("god_mode_enabled")? != 0;
  let hell_mode_enabled = read::byte(loadstate).context("hell_mode_enabled")? != 0;

  let mut lua_keys = Vec::new();
  let size = read::u32(loadstate).context("lua_keys size")?;
  for _ in 0..size {
    let lua_key = read_string(loadstate).context("lua_key")?;
    lua_keys.push(lua_key);
  }

  let current_map_name = read_string(loadstate).context("current_map_name")?;
  let start_next_map = read_string(loadstate).context("start_next_map")?;
  let lua_state_size = read::u32(loadstate).context("lua_state size")?;
  let lua_state_lz4 = read::bytes(loadstate, lua_state_size.try_into().unwrap()).context("lua_state bytes")?;
  
  let lua_state = lz4::block::decompress(
    &lua_state_lz4,
    Some(HadesSaveV16::UNCOMPRESSED_SIZE))?;

  Ok(HadesSaveV16 {
    timestamp: timestamp,
    location: location,
    runs: runs,
    active_meta_points: active_meta_points,
    active_shrine_points: active_shrine_points,
    god_mode_enabled: god_mode_enabled,
    hell_mode_enabled: hell_mode_enabled,
    lua_keys: lua_keys,
    current_map_name: current_map_name,
    start_next_map: start_next_map,
    lua_state: lua_state
  })
}

pub fn read_v17(loadstate: &mut &[u8]) -> Result<HadesSaveV17> {
  let timestamp = read::u64(loadstate).context("timestamp")?;
  let location = read_string(loadstate).context("location")?;
  let padding1 = read::bytes(loadstate, 12).context("padding1")?.to_vec();
  let runs = read::u32(loadstate).context("runs")?;
  let god_mode_enabled = read::byte(loadstate).context("god_mode_enabled")? != 0;
  let hell_mode_enabled = read::byte(loadstate).context("hell_mode_enabled")? != 0;

  let mut lua_keys = Vec::new();
  let size = read::u32(loadstate).context("lua_keys size")?;
  for _ in 0..size {
    let lua_key = read_string(loadstate).context("lua_key")?;
    lua_keys.push(lua_key);
  }

  let current_map_name = read_string(loadstate).context("current_map_name")?;
  let start_next_map = read_string(loadstate).context("start_next_map")?;
  let lua_state_size = read::u32(loadstate).context("lua_state size")?;
  let lua_state_lz4 = read::bytes(loadstate, lua_state_size.try_into().unwrap()).context("lua_state bytes")?;
  
  let lua_state = lz4::block::decompress(
    &lua_state_lz4,
    Some(HadesSaveV16::UNCOMPRESSED_SIZE))?;

  let lua_size = luabins::size(lua_state.as_slice())?;
  println!(
    "uncompressed {} luasize {}",
    lua_state.len(),
    lua_size);

  Ok(HadesSaveV17 {
    timestamp: timestamp,
    location: location,
    padding1: padding1,
    runs: runs,
    god_mode_enabled: god_mode_enabled,
    hell_mode_enabled: hell_mode_enabled,
    lua_keys: lua_keys,
    current_map_name: current_map_name,
    start_next_map: start_next_map,
    lua_state: lua_state[0..lua_size].to_vec()
  })
}

fn write_string(contents: &mut Vec<u8>, string: &str) {
  write::u32(contents, string.len() as u32);
  let mut str_bytes = string.as_bytes().to_owned();
  write::bytes(contents, &mut str_bytes);
}

pub fn write (save: &HadesSave) -> Result<Vec<u8>> {
  match save {
    HadesSave::V16(save) => return write_v16(save),
    HadesSave::V17(save) => return write_v17(save)
  }
}

pub fn write_v16 (save: &HadesSaveV16) -> Result<Vec<u8>> {
  let mut contents: Vec<u8> = Vec::new();
  let mut signature = "SGB1".as_bytes().to_owned();
  write::bytes(&mut contents, &mut signature);
  let mut checksum = "TODO".as_bytes().to_owned();
  write::bytes(&mut contents, &mut checksum);
  write::u32(&mut contents, 16); // version
  write::u64(&mut contents, save.timestamp);
  write_string(&mut contents, &save.location);
  write::u32(&mut contents, save.runs);
  write::u32(&mut contents, save.active_meta_points);
  write::u32(&mut contents, save.active_shrine_points);
  write::byte(&mut contents, if save.god_mode_enabled {1} else {0});
  write::byte(&mut contents, if save.hell_mode_enabled {1} else {0});

  // lua keys
  write::u32(&mut contents, save.lua_keys.len() as u32);
  for lua_key in save.lua_keys.iter() {
    write_string(&mut contents, lua_key)
  }

  write_string(&mut contents, &save.current_map_name);
  write_string(&mut contents, &save.start_next_map);

  let mut lua_state_lz4 = lz4::block::compress(&save.lua_state, None, false)?;
  write::u32(&mut contents, lua_state_lz4.len() as u32);
  write::bytes(&mut contents, &mut lua_state_lz4);

  let checksum_bytes = adler32(&contents[8..])?.to_ne_bytes();
  contents[4] = checksum_bytes[0];
  contents[5] = checksum_bytes[1];
  contents[6] = checksum_bytes[2];
  contents[7] = checksum_bytes[3];

  Ok(contents)
}


pub fn write_v17 (save: &HadesSaveV17) -> Result<Vec<u8>> {
  let mut contents: Vec<u8> = Vec::new();
  let mut signature = "SGB1".as_bytes().to_owned();
  write::bytes(&mut contents, &mut signature);
  let mut checksum = "TODO".as_bytes().to_owned();
  write::bytes(&mut contents, &mut checksum);
  write::u32(&mut contents, 17); // version
  write::u64(&mut contents, save.timestamp);
  write_string(&mut contents, &save.location);
  write::bytes(&mut contents, &mut save.padding1.clone());
  write::u32(&mut contents, save.runs);
  write::byte(&mut contents, if save.god_mode_enabled {1} else {0});
  write::byte(&mut contents, if save.hell_mode_enabled {1} else {0});

  // lua keys
  write::u32(&mut contents, save.lua_keys.len() as u32);
  for lua_key in save.lua_keys.iter() {
    write_string(&mut contents, lua_key)
  }

  write_string(&mut contents, &save.current_map_name);
  write_string(&mut contents, &save.start_next_map);

  let lua_size = luabins::size(&save.lua_state)?;
  println!("lua_state.len() {} lua_size {}", save.lua_state.len(), lua_size);

  let mut lua_state_lz4 = lz4::block::compress(&save.lua_state, None, false)?;

  write::u32(&mut contents, lua_state_lz4.len() as u32);
  write::bytes(&mut contents, &mut lua_state_lz4);

  let checksum_bytes = adler32(&contents[8..])?.to_ne_bytes();
  contents[4] = checksum_bytes[0];
  contents[5] = checksum_bytes[1];
  contents[6] = checksum_bytes[2];
  contents[7] = checksum_bytes[3];

  Ok(contents)
}