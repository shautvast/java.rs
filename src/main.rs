use std::io::Error;
use java_rs::vm::Vm;

fn main() -> Result<(), Error> {
    // TODO cmdline args
    // TODO build index for package -> jarfile?

    let mut vm = Vm::new("tests");
    // let main_class = "Inheritance";
    let main_class = "testclasses.Main";

    vm.execute_static( main_class, "main([Ljava/lang/String;)V", vec![])
        .unwrap();
    Ok(())
}
