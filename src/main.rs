use java_rs::vm::runtime::Vm;

fn main() {
    let a = 0.0;
    println!("{}", 1.0 / a);
    let mut vm = Vm::new();
    vm.run(
        "/Users/Shautvast/dev/java.rs/tests",
        "testclasses/Main",
        "main([Ljava/lang/String;)V",
    );
}
