use std::fs::{self};

use anyhow::{anyhow, Error};
use crate::vm::opcodes::Opcode;
use crate::vm::opcodes::Opcode::*;

#[cfg(target_family = "unix")]
pub const PATH_SEPARATOR: char = ':';

#[cfg(target_family = "windows")]
pub const PATH_SEPARATOR: char = ';';

/// resolves the actual path where the class file is found
/// for std lib there is a  special case that resolves to the jmod
/// notation:
/// * [jmod]#classes/[package_path]/[class].class
/// * [jar/zip]#[package_path]/[class].class
/// * [dir]/[package_path]/[class].class
pub fn find_class(classpath: &Vec<String>, class_name: &str) -> Result<String, Error> {
    let class_name = &class_name.to_owned().replace(".", "/");
    if class_name.starts_with("java")
        || class_name.starts_with("sun/")
        || class_name.starts_with("com/sun/")
        || class_name.starts_with("jdk/")
    {
        let mut path: String = "jmods/java.base.jmod#classes/".into();
        path.push_str(class_name);
        path.push_str(".class");
        return Ok(path);
    }

    for clp_entry in classpath {
        if fs::metadata(clp_entry)?.is_dir() {
            let mut maybe_path = clp_entry.clone();
            maybe_path.push('/');
            maybe_path.push_str(class_name);
            maybe_path.push_str(".class");
            if fs::metadata(&maybe_path)?.is_file() {
                return Ok(maybe_path);
            }
        } else {
            //TODO jar/zip files
        }
    }
    Err(anyhow!("Class not found {}", class_name))
}


// methods to read values from big-endian binary data

pub(crate) fn read_u8(data: &[u8], pos: &mut usize) -> u8 {
    *pos += 1;
    u8::from_be_bytes(
        data[*pos - 1..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_bytes(data: &[u8], pos: &mut usize, len: usize) -> Vec<u8> {
    *pos += len;
    data[*pos - len..*pos]
        .try_into()
        .expect("slice with incorrect length")
}

pub(crate) fn read_u16(data: &[u8], pos: &mut usize) -> u16 {
    *pos += 2;
    u16::from_be_bytes(
        data[*pos - 2..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_i32(data: &[u8], pos: &mut usize) -> i32 {
    *pos += 4;
    i32::from_be_bytes(
        data[*pos - 4..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_u32(data: &[u8], pos: &mut usize) -> u32 {
    *pos += 4;
    u32::from_be_bytes(
        data[*pos - 4..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_f32(data: &[u8], pos: &mut usize) -> f32 {
    *pos += 4;
    f32::from_be_bytes(
        data[*pos - 4..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_i64(data: &[u8], pos: &mut usize) -> i64 {
    *pos += 8;
    i64::from_be_bytes(
        data[*pos - 8..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_f64(data: &[u8], pos: &mut usize) -> f64 {
    *pos += 8;
    f64::from_be_bytes(
        data[*pos - 8..*pos]
            .try_into()
            .expect("slice with incorrect length"),
    )
}

pub(crate) fn read_tableswitch(data: &[u8], pos: &mut usize) -> Tableswitch {
    while read_u8(data, pos) == 0 {}
    *pos -= 1;
    let default = read_i32(data, pos);
    let low = read_i32(data, pos);
    let high = read_i32(data, pos);
    let mut offsets = vec![];
    for _ in low..=high {
        offsets.push(read_i32(data, pos));
    }
    Tableswitch { default, low, high, offsets }
}

pub(crate) fn read_lookupswitch(data: &[u8], pos: &mut usize) -> Lookupswitch {
    while read_u8(data, pos) == 0 {}
    *pos -= 1;
    let default = read_i32(data, pos);
    let npairs = read_i32(data, pos);
    let mut match_offset_pairs = vec![];
    for _ in 0..npairs {
        match_offset_pairs.push((read_i32(data, pos), read_i32(data, pos)));
    }
    Lookupswitch { default, match_offset_pairs }
}

pub(crate) fn read_wide_opcode(data: &[u8], pos: &mut usize) -> Opcode {
    let opcode = read_u8(data, pos);
    if opcode == 132 {
        WIDE_IINC(read_u16(data, pos), read_u16(data, pos))
    } else {
        let index = read_u16(data, pos);
        match opcode {
            21 => WIDE_ILOAD(index),
            22 => WIDE_LLOAD(index),
            23 => WIDE_FLOAD(index),
            24 => WIDE_DLOAD(index),
            25 => WIDE_ALOAD(index),
            54 => WIDE_ISTORE(index),
            55 => WIDE_LSTORE(index),
            56 => WIDE_FSTORE(index),
            57 => WIDE_DSTORE(index),
            58 => WIDE_ASTORE(index),
            169 => WIDE_RET(index),
            _ => { unreachable!("unknown opcode for WIDE") }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Tableswitch {
    default: i32,
    low: i32,
    high: i32,
    offsets: Vec<i32>,
}

#[derive(Clone, Debug)]
pub(crate) struct Lookupswitch {
    default: i32,
    match_offset_pairs: Vec<(i32, i32)>,
}