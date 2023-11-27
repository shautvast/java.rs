use java_rs::vm::runtime::Vm;

fn main() {
    let mut vm = Vm::new();
    vm.run(
        "/Users/Shautvast/dev/java.rs/tests",
        "testclasses/Main",
        "main([Ljava/lang/String;)V",
    );
}
