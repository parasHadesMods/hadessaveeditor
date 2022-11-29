use crate::read;
use crate::write;

use adler32::adler32;
use anyhow::{bail, Context, Result};
use std::convert::TryInto;

pub trait UncompressedSize {
  const UNCOMPRESSED_SIZE: i32;
}

pub struct HadesSaveV16 {
  pub version: u32,
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
  pub lua_state_lz4: Vec<u8>
}

impl UncompressedSize for HadesSaveV16 {
  const UNCOMPRESSED_SIZE: i32 = 9388032;
}


fn read_string(loadstate: &mut &[u8]) -> Result<String> {
  let size = read::u32(loadstate).context("size")?;
  let str_bytes = read::bytes(loadstate, size.try_into().unwrap()).context("bytes")?;
  String::from_utf8(str_bytes.to_vec()).context("utf8")
}

pub fn read(loadstate: &mut &[u8]) -> Result<HadesSaveV16> {
  let signature = read::bytes(loadstate, 4).context("signature")?;
  if signature != "SGB1".as_bytes() {
    bail!("Not a Hades save file");
  }
  let _checksum = read::bytes(loadstate, 4).context("checksum")?;
  let version = read::u32(loadstate).context("version")?;
  if version != 16 {
    bail!("unknown version {}", version);
  };
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
  
  Ok(HadesSaveV16 {
    version: version,
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
    lua_state_lz4: lua_state_lz4.to_vec()
  })
}

fn write_string(contents: &mut Vec<u8>, string: &str) {
  write::u32(contents, string.len() as u32);
  let mut str_bytes = string.as_bytes().to_owned();
  write::bytes(contents, &mut str_bytes);
}

pub fn write (save: &HadesSaveV16) -> Result<Vec<u8>> {
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

  write::u32(&mut contents, save.lua_state_lz4.len() as u32);
  write::bytes(&mut contents, save.lua_state_lz4.clone().as_mut_slice());

  let checksum_bytes = adler32(&contents[8..])?.to_ne_bytes();
  contents[4] = checksum_bytes[0];
  contents[5] = checksum_bytes[1];
  contents[6] = checksum_bytes[2];
  contents[7] = checksum_bytes[3];

  Ok(contents)
}
