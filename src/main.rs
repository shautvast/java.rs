use std::fs::{self, File};
use std::io::Read;

fn main() {
    let bytecode = read_class_file("./Dummy.class");
    if let Some(class) = classfile_reader::get_class(bytecode){
        let ret = class.execute("public static get()I");
        println!("{:?}", ret);
    }

}

fn read_class_file(name: &str) -> Vec<u8> {
    let mut f = File::open(name).expect("no file found");
    let metadata = fs::metadata(name).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    let _ = f.read(&mut buffer).expect("buffer overflow");
    buffer
}
