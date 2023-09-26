mod test {
    use classfile_reader::{get_class, io};
    use classfile_reader::CpEntry::*;
    use classfile_reader::types::Value;

    #[test]
    fn get_constant_int() {
        let class = get_class(io::read_class_file("tests/Int.class")).unwrap();
        assert_eq!((55, 0), class.get_version());
        if let Value::I32(v) = class.methods.get("public static get()I").unwrap().execute().unwrap() {
            assert_eq!(v, 42);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn get_constant_double() {
        let class = get_class(io::read_class_file("tests/Double.class")).unwrap();
        assert_eq!((55, 0), class.get_version());
        if let Value::F64(v) = class.methods.get("public static get()D").unwrap().execute().unwrap() {
            assert_eq!(v, 42.0);
        } else {
            panic!("fail");
        }
    }
}