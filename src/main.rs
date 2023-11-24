use java_rs::vm::runtime::Vm;

fn main() {
    let vm = Vm::new();
    vm.run("/Users/Shautvast/dev/java/tests", "testclasses.Main", "main([Ljava/lang/String;)V");
}

