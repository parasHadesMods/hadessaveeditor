use std::convert::TryInto;
use anyhow::{anyhow, Result};

pub fn byte(loadstate: &mut &[u8]) -> Result<u8> {
  match loadstate.split_first() {
    Some((first, rest)) => {
      *loadstate = rest;
      Ok(*first)
    },
    None => Err(anyhow!("read::byte failed, not enough data (needed 1 byte, 0 available)"))
  }
}

pub fn i32(loadstate: &mut &[u8]) -> Result<i32> {
    if loadstate.len() >= 4 {
        let (i32_bytes, rest) = loadstate.split_at(4);
        *loadstate = rest;
        Ok(i32::from_ne_bytes(i32_bytes.try_into().unwrap()))
    } else {
        Err(anyhow!("read::i32 failed, not enough data (needed 4 bytes, {} available)", loadstate.len()))
    }
}

pub fn u32(loadstate: &mut &[u8]) -> Result<u32> {
    if loadstate.len() >= 4 {
        let (u32_bytes, rest) = loadstate.split_at(4);
        *loadstate = rest;
        Ok(u32::from_ne_bytes(u32_bytes.try_into().unwrap()))
    } else {
        Err(anyhow!("read::u32 failed, not enough data (needed 4 bytes, {} available)", loadstate.len()))
    }
}

pub fn u64(loadstate: &mut &[u8]) -> Result<u64> {
    if loadstate.len() >= 8 {
        let (u64_bytes, rest) = loadstate.split_at(8);
        *loadstate = rest;
        Ok(u64::from_ne_bytes(u64_bytes.try_into().unwrap()))
    } else {
        Err(anyhow!("read::u64 failed, not enough data (needed 8 bytes, {} available)", loadstate.len()))
    }
}

pub fn f64(loadstate: &mut &[u8]) -> Result<f64> {
    if loadstate.len() >= 8 {
        let (f64_bytes, rest) = loadstate.split_at(8);
        *loadstate = rest;
        Ok(f64::from_ne_bytes(f64_bytes.try_into().unwrap()))
    } else {
        Err(anyhow!("read::f64 failed, not enough data (needed 8 bytes, {} available)", loadstate.len()))
    }
}

pub fn bytes<'a>(loadstate: &'a mut &[u8], len: usize) -> Result<&'a [u8]> {
    if loadstate.len() >= len {
        let (bytes, rest) = loadstate.split_at(len);
        *loadstate = rest;
        Ok(bytes)
    } else {
        Err(anyhow!("read::bytes failed, not enough data (needed {} bytes, {} available)", len, loadstate.len()))
    }
}
