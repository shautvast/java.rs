use std::io::Error;
use classfile_reader::vm::Vm;

fn main() -> Result<(), Error> {
    let mut vm = Vm::new("tests");
    vm.execute("Main","public static main([Ljava/lang/String;)V", None).unwrap();
    Ok(())
}


