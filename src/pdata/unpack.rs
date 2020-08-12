use anyhow::{anyhow, Result};
use nom::{number::complete::*, IResult};

//------------------------------------------

pub trait Unpack {
    // The size of the value when on disk.
    fn disk_size() -> u32;
    fn unpack(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: std::marker::Sized;
}

pub fn unpack<U: Unpack>(data: &[u8]) -> Result<U> {
    match U::unpack(data) {
        Err(_e) => Err(anyhow!("couldn't parse SMRoot")),
        Ok((_i, v)) => Ok(v),
    }
}

//------------------------------------------

impl Unpack for u64 {
    fn disk_size() -> u32 {
        8
    }

    fn unpack(i: &[u8]) -> IResult<&[u8], u64> {
        le_u64(i)
    }
}

impl Unpack for u32 {
    fn disk_size() -> u32 {
        4
    }

    fn unpack(i: &[u8]) -> IResult<&[u8], u32> {
        le_u32(i)
    }
}

//------------------------------------------
