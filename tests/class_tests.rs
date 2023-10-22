mod test {
    use java_rs::class::{get_class, Value};
    use java_rs::heap::ObjectRef;
    use java_rs::vm::Vm;
    #[test]
    fn if_cmp() {
        let mut vm = Vm::new("tests");
        let ret = vm.execute("testclasses.IfCmp", "i_is_1()Z", vec![]).unwrap();
        unsafe {
            if let Value::I32(b) = *ret.get() {
                // internally a boolean is an int
                assert_eq!(0, b);
            } else {
                println!("{:?}", *ret.get());
                assert!(false)
            }
        }
    }

    #[test]
    fn consts() {
        let mut vm = Vm::new("tests");
        let ret = vm
            .execute("testclasses.Const", "hello()Ljava/lang/String;", vec![])
            .unwrap();
        unsafe {
            if let Value::Ref(s) = &*ret.get() {
                // internally a boolean is an int
                if let ObjectRef::Object(a) = &*s.get() {
                    println!("{:?}", a);
                }
            } else {
                println!("{:?}", *ret.get());
                assert!(false)
            }
        }
    }
}
