use anyhow::{anyhow, Error};
use std::fs::{self, File};
use std::io::Read;

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
            // println!("{}", maybe_path);
            if fs::metadata(&maybe_path)?.is_file() {
                return Ok(maybe_path);
            }
        } else {
            //TODO jar/zip files
        }
    }
    Err(anyhow!("Class not found {}", class_name))
}

/// reads the binary class file from file path or archive
/// and returns the byte array as Vec
pub fn read_bytecode(name: String) -> Result<Vec<u8>, Error> {
    let mut buffer;
    if name.contains('#') {
        let parts: Vec<&str> = name.split('#').collect();
        let archive_file = File::open(parts[0])?;
        let mut archive_zip = zip::ZipArchive::new(archive_file)?;
        let mut entry = archive_zip.by_name(parts[1])?;
        buffer = vec![0; entry.size() as usize];
        entry.read_exact(&mut buffer)?;
    } else {
        let mut f = File::open(&name)?;
        let metadata = fs::metadata(&name)?;
        buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer)?;
    }
    Ok(buffer)
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
