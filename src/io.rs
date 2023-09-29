use std::fs::{self, File};
use std::io::Read;
use anyhow::{anyhow, Error};

pub(crate) fn read_u8(data: &[u8], pos: usize) -> u8 {
    u8::from_be_bytes(data[pos..pos + 1].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_u16(data: &[u8], pos: usize) -> u16 {
    u16::from_be_bytes(data[pos..pos + 2].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_i32(data: &[u8], pos: usize) -> i32 {
    i32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_u32(data: &[u8], pos: usize) -> u32 {
    u32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_f32(data: &[u8], pos: usize) -> f32 {
    f32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_i64(data: &[u8], pos: usize) -> i64 {
    i64::from_be_bytes(data[pos..pos + 8].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_f64(data: &[u8], pos: usize) -> f64 {
    f64::from_be_bytes(data[pos..pos + 8].try_into().expect("slice with incorrect length"))
}

pub fn find_class(classpath: &Vec<String>, class_name: &str) -> Result<String, Error> {
    for clp_entry in classpath {
        let mut maybe_path = clp_entry.clone();
        maybe_path.push('/');
        maybe_path.push_str(class_name);
        maybe_path.push_str(".class");
        println!("{}", maybe_path);
        if fs::metadata(&maybe_path)?.is_file() {
            return Ok(maybe_path);
        }
    }
    Err(anyhow!("Class not found {}", class_name))
}

pub fn read_class_file(name: String) -> Result<Vec<u8>, Error> {
    let mut f = File::open(&name)?;
    let metadata = fs::metadata(&name)?;
    let mut buffer = vec![0; metadata.len() as usize];
    let _ = f.read(&mut buffer)?;
    Ok(buffer)
}