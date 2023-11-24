mod test {
    use java_rs::vm::runtime::Vm;



    #[test]
    fn if_cmp() {
        let vm = Vm::new();
        vm.run("/Users/Shautvast/dev/java/tests", "testclasses.IfCmp", "i_is_1()Z");
    }

    #[test]
    fn consts() {

        let vm = Vm::new();
        vm.run("/Users/Shautvast/dev/java/tests", "testclasses.Const", "hello()Ljava/lang/String;")
    }
}
