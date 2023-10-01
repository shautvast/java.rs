use java_rs::vm::Vm;
use std::io::Error;

fn main() -> Result<(), Error> {
    // TODO cmdline args
    // TODO build index for package -> jarfile?

    let mut vm = Vm::new("tests");
    let main_class = "Main";

    vm.execute(main_class, "main([Ljava/lang/String;)V", vec![])
        .unwrap();
    Ok(())
}
