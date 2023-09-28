mod test {
    use std::rc::Rc;
    use classfile_reader::{classloader::load_class, io};
    use classfile_reader::class::Value;
    use classfile_reader::vm::Vm;

    #[test]
    fn get_constant_int() {
        let mut vm = Vm::new(".");
        let class = vm.get_class("Float").expect("ClassNotFound");
        assert_eq!((55, 0), class.get_version());


        if let Value::I32(v) = Vm::new("").execute("Float", "public static get()I").unwrap() {
            assert_eq!(v, 42);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn get_constant_double() {
        let mut vm = Vm::new(".");
        let class = vm.get_class("Double").expect("ClassNotFound");
        assert_eq!((55, 0), class.get_version());
        if let Value::F64(v) = Vm::new("").execute("Double", "public static get()D").unwrap() {
            assert_eq!(v, 42.0);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn get_constant_foat() {
        let mut vm = Vm::new(".");
        vm.load_class("Float").expect("ClassNotFound");
        // assert_eq!((55, 0), class.get_version());
        // if let Value::F32(v) = Vm::new().execute(class.methods.get("public static getF()F").unwrap()).unwrap() {
        //     assert_eq!(v, 42.0);
        // } else {
        //     panic!("fail");
        // }
    }

    #[test]
    fn get_float() {
        // assert_eq!((55, 0), class.get_version());
        let mut vm = Vm::new("/Users/FJ19WK/RustroverProjects/classfile_reader/tests");
        vm.load_class("Float").expect("ClassNotFound");
        if let Value::F32(v) = vm.execute("Float","public getF2()F").unwrap() {
            assert_eq!(v, 0.0);
        } else {
            panic!("fail");
        }
    }
}