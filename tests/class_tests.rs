mod test {
    use std::rc::Rc;
    use classfile_reader::CpEntry::*;
    use classfile_reader::types::{AttributeType, Class, Field, Method};

    #[test]
    fn get_version() {
        assert_eq!((55, 0), get_class().get_version());
    }

    #[test]
    fn get_methods() {
        let class = get_class();
        assert_eq!("public <init>(Ljava/lang/String;)V", &class.methods[0].name());
        assert_eq!("public getName()Ljava/lang/String;", &class.methods[1].name());
        assert_eq!("public print()V", &class.methods[2].name());
    }

    #[test]
    fn get_fields() {
        let class = get_class();
        assert_eq!("private final Ljava/lang/String; name", &class.fields[0].name())
    }

    #[test]
    fn get_code() {
        let class = get_class();
        // println!("{:?}", &class.methods[0].get_code());
        // println!("{:?}", &class.methods[1].get_code());
    }

    fn get_class() -> Class {
        let cp = Rc::new(vec![
            MethodRef(1, 6, 19),
            Fieldref(2, 5, 20),
            Fieldref(3, 21, 22),
            MethodRef(4, 23, 24),
            ClassRef(5, 25),
            ClassRef(6, 26),
            Utf8(7, "name".to_owned()),
            Utf8(8, "Ljava/lang/String;".to_owned()),
            Utf8(9, "<init>".to_owned()),
            Utf8(10, "(Ljava/lang/String;)V".to_owned()), //10
            Utf8(11, "Code".to_owned()),
            Utf8(12, "LineNumberTable".to_owned()),
            Utf8(13, "getName".to_owned()),
            Utf8(14, "()Ljava/lang/String;".to_owned()),
            Utf8(15, "print".to_owned()),
            Utf8(16, "()V".to_owned()),
            Utf8(17, "SourceFile".to_owned()),
            Utf8(18, "Dummy.java".to_owned()),
            NameAndType(19, 9, 16),
            NameAndType(20, 7, 8), //20
            ClassRef(21, 27),
            NameAndType(22, 28, 29),
            ClassRef(23, 30),
            NameAndType(24, 31, 10),
            Utf8(25, "dummy/Dummy".to_owned()),
            Utf8(26, "java/lang/Object".to_owned()),
            Utf8(27, "java/lang/System".to_owned()),
            Utf8(28, "out".to_owned()),
            Utf8(29, "Ljava/io/PrintStream;".to_owned()),
            Utf8(30, "java/io/PrintStream".to_owned()),
            Utf8(31, "println".to_owned()),
        ]);

        Class {
            minor_version: 0,
            major_version: 55,
            constant_pool: cp.clone(),
            access_flags: 33,
            this_class: 5,
            super_class: 6,
            interfaces: vec![],
            methods: vec![
                Method::new(
                    cp.clone(), 1, 9, 10, vec![AttributeType::Deprecated
                    ],
                ),
                Method::new(
                    cp.clone(), 1, 13, 14, vec![AttributeType::Deprecated]),
                Method::new(cp.clone(), 1, 15, 16, vec![AttributeType::Deprecated]),
            ],
            fields: vec![Field::new(cp, 18, 7, 8, vec![])],
            attributes: vec![AttributeType::SourceFile],
        }
    }
}