mod test {
    use classfile_reader::{get_class, io};
    use classfile_reader::class::Value;
    use classfile_reader::vm::Vm;

    #[test]
    fn get_constant_int() {
        let class = get_class(io::read_class_file("tests/Int.class")).unwrap();
        assert_eq!((55, 0), class.get_version());


        if let Value::I32(v) = Vm::new().execute(class.methods.get("public static get()I").unwrap()).unwrap() {
            assert_eq!(v, 42);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn get_constant_double() {
        let class = get_class(io::read_class_file("tests/Double.class")).unwrap();
        assert_eq!((55, 0), class.get_version());
        if let Value::F64(v) = Vm::new().execute(class.methods.get("public static get()D").unwrap()).unwrap() {
            assert_eq!(v, 42.0);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn get_constant_foat() {
        let class = get_class(io::read_class_file("tests/Float.class")).unwrap();
        Vm::new().new_instance(class);
        // assert_eq!((55, 0), class.get_version());
        // if let Value::F32(v) = Vm::new().execute(class.methods.get("public static getF()F").unwrap()).unwrap() {
        //     assert_eq!(v, 42.0);
        // } else {
        //     panic!("fail");
        // }
    }

    #[test]
    fn get_foat() {
        let class = get_class(io::read_class_file("tests/Float.class")).unwrap();
        assert_eq!((55, 0), class.get_version());
        if let Value::F32(v) = Vm::new().execute(class.methods.get("public getF2()F").unwrap()).unwrap() {
            assert_eq!(v, 42.0);
        } else {
            panic!("fail");
        }
    }
}