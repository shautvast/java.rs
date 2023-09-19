use std::fs::{self, File};
use std::io::Read;

fn main() {
    let bytecode = read_class_file("/Users/FJ19WK/RustroverProjects/classfile_reader/MetaField.class");
    if let Some(class) = classfile_reader::get_class(bytecode){
        println!("{:?}", class);
    }
}

fn read_class_file(name: &str) -> Vec<u8> {
    let mut f = File::open(name).expect("no file found");
    let metadata = fs::metadata(name).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}
