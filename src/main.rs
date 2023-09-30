use java_rs::vm::Vm;
use std::io::Error;

fn main() -> Result<(), Error> {
    let mut vm = Vm::new("tests");
    vm.execute("Main", "public static main([Ljava/lang/String;)V", None)
        .unwrap();
    Ok(())
}
