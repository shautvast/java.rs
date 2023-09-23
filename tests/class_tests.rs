mod test {
    use std::rc::Rc;
    use classfile_reader::CpEntry::*;
    use classfile_reader::types::{Attribute, Class, Field, Method};

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
        println!("{:?}", &class.methods[0].get_code());
        println!("{:?}", &class.methods[1].get_code());
    }

    fn get_class() -> Class {
        let cp = Rc::new(vec![
            MethodRef(6, 19),
            Fieldref(5, 20),
            Fieldref(21, 22),
            MethodRef(23, 24),
            ClassRef(25),
            ClassRef(26),
            Utf8("name".to_owned()),
            Utf8("Ljava/lang/String;".to_owned()),
            Utf8("<init>".to_owned()),
            Utf8("(Ljava/lang/String;)V".to_owned()), //10
            Utf8("Code".to_owned()),
            Utf8("LineNumberTable".to_owned()),
            Utf8("getName".to_owned()),
            Utf8("()Ljava/lang/String;".to_owned()),
            Utf8("print".to_owned()),
            Utf8("()V".to_owned()),
            Utf8("SourceFile".to_owned()),
            Utf8("Dummy.java".to_owned()),
            NameAndType(9, 16),
            NameAndType(7, 8), //20
            ClassRef(27),
            NameAndType(28, 29),
            ClassRef(30),
            NameAndType(31, 10),
            Utf8("dummy/Dummy".to_owned()),
            Utf8("java/lang/Object".to_owned()),
            Utf8("java/lang/System".to_owned()),
            Utf8("out".to_owned()),
            Utf8("Ljava/io/PrintStream;".to_owned()),
            Utf8("java/io/PrintStream".to_owned()),
            Utf8("println".to_owned()),
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
                    cp.clone(), 1, 9, 10, vec![Attribute {
                        attribute_name_index: 11,
                        info: vec![0, 2, 0, 2, 0, 0, 0, 10, 42, 183, 0, 1, 42, 43, 181, 0, 2, 177, 0,
                                   0, 0, 1, 0, 12, 0, 0, 0, 14, 0, 3, 0, 0, 0, 7, 0, 4, 0, 8, 0, 9, 0, 9],
                    }],
                ),
                Method::new(
                    cp.clone(), 1, 13, 14, vec![Attribute {
                        attribute_name_index: 11,
                        info: vec![0, 1, 0, 1, 0, 0, 0, 5, 42, 180, 0, 2, 176, 0, 0, 0, 1, 0, 12, 0,
                                   0, 0, 6, 0, 1, 0, 0, 0, 12],
                    }],
                ),
                Method::new(cp.clone(), 1, 15, 16, vec![Attribute {
                    attribute_name_index: 11,
                    info: vec![0, 2, 0, 1, 0, 0, 0, 11, 178, 0, 3, 42, 180, 0, 2, 182, 0, 4, 177, 0,
                               0, 0, 1, 0, 12, 0, 0, 0, 10, 0, 2, 0, 0, 0, 16, 0, 10, 0, 17],
                }]),
            ],
            fields: vec![Field::new(cp, 18, 7, 8, vec![])],
            attributes: vec![Attribute { attribute_name_index: 17, info: vec![0, 18] }],
        }
    }
}