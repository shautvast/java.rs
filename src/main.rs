use java_rs::classmanager::set_classpath;
use java_rs::vm::Vm;

fn main() {
    let mut stackframes = Vec::new();
    let mut vm = Vm::new(&mut stackframes);
    set_classpath("/Users/Shautvast/dev/java/tests");
    let main_class = "testclasses.Main";
    vm.execute_static( &mut stackframes, main_class, "main([Ljava/lang/String;)V", vec![])
        .unwrap();
}
