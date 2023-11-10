mod test {
    use java_rs::class::{ObjectRef, Value};
    use java_rs::classmanager::set_classpath;
    use java_rs::vm::Vm;

    #[test]
    fn if_cmp() {
        let mut stackframes = Vec::new();
        let mut vm = Vm::new(&mut stackframes);
        set_classpath("/Users/Shautvast/dev/java/tests");
        let ret = vm.execute_virtual(&mut stackframes, "testclasses.IfCmp", "i_is_1()Z", vec![]).unwrap();
        if let Value::I32(b) = ret {
            // internally a boolean is an int
            assert_eq!(0, b);
        } else {
            println!("{:?}", ret);
            assert!(false)
        }
    }

    #[test]
    fn consts() {
        let mut stackframes = Vec::new();
        let mut vm = Vm::new(&mut stackframes);
        set_classpath("/Users/Shautvast/dev/java/tests");
        let ret = vm
            .execute_static(&mut stackframes, "testclasses.Const", "hello()Ljava/lang/String;", vec![])
            .unwrap();
        if let Value::Ref(s) = ret {
            // internally a boolean is an int
            if let ObjectRef::Object(a) = s {
                println!("{:?}", a);
            }
        } else {
            println!("{:?}", ret);
            assert!(false)
        }
    }
}
