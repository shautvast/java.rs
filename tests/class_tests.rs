mod test {
    use std::rc::Rc;
    use classfile_reader::types::{Attribute, Class, Field, Method};
    use classfile_reader::types::CpEntry::*;

    #[test]
    fn get_version() {
        assert_eq!((55, 0), get_class().get_version());
    }

    #[test]
    fn get_methods() {
        for m in get_class().get_methods() {
            println!("{}", m.name());
        }
    }

    fn get_class() -> Class {
        let cp = Rc::new(vec![MethodRef(2, 3),
                              ClassRef(4),
                              NameAndType(5, 6),
                              Utf8("java/lang/Object".to_owned()),
                              Utf8("<init>".to_owned()),
                              Utf8("()V".to_owned()),
                              Fieldref(8, 9),
                              ClassRef(10),
                              NameAndType(11, 12),
                              Utf8("com/github/shautvast/reflective/MetaField".to_owned()),
                              Utf8("name".to_owned()),
                              Utf8("Ljava/lang/String;".to_owned()),
                              Fieldref(8, 14),
                              NameAndType(15, 16),
                              Utf8("modifiers".to_owned()),
                              Utf8("I".to_owned()),
                              Utf8("(Ljava/lang/String;I)V".to_owned()),
                              Utf8("Code".to_owned()),
                              Utf8("LineNumberTable".to_owned()),
                              Utf8("LocalVariableTable".to_owned()),
                              Utf8("this".to_owned()),
                              Utf8("Lcom/github/shautvast/reflective/MetaField;".to_owned()),
                              Utf8("getName".to_owned()),
                              Utf8("()Ljava/lang/String;".to_owned()),
                              Utf8("getModifiers".to_owned()),
                              Utf8("()I".to_owned()),
                              Utf8("SourceFile".to_owned()),
                              Utf8("MetaField.java".to_owned())]);

        Class {
            minor_version: 0,
            major_version: 55,
            constant_pool: cp.clone(),
            interfaces: vec![],
            super_class: 2,
            access_flags: 33,
            this_class: 8,
            methods: vec![
                Method {
                    constant_pool: cp.clone(),
                    access_flags: 1,
                    name_index: 5,
                    descriptor_index: 17,
                    attributes_count: 1,
                    attributes: vec![Attribute {
                        attribute_name_index: 18,
                        info: vec![0, 2, 0, 3, 0, 0, 0, 15, 42, 183, 0, 1, 42, 43, 181, 0, 7, 42, 28,
                                   181, 0, 13, 177, 0, 0, 0, 2, 0, 19, 0, 0, 0,
                                   18, 0, 4, 0, 0, 0, 8, 0, 4, 0, 9, 0, 9, 0, 10, 0, 14, 0, 11, 0, 20, 0, 0,
                                   0, 32, 0, 3, 0, 0, 0, 15, 0, 21, 0, 22, 0, 0, 0, 0, 0, 15, 0, 11,
                                   0, 12, 0, 1, 0, 0, 0, 15, 0, 15, 0, 16, 0, 2],
                    }],
                },
                Method {
                    constant_pool: cp,
                    access_flags: 1,
                    name_index: 23,
                    descriptor_index: 24,
                    attributes_count: 1,
                    attributes: vec![Attribute {
                        attribute_name_index: 18,
                        info: vec![0, 1, 0, 1, 0, 0, 0, 5, 42, 180, 0, 7, 176, 0, 0, 0, 2, 0, 19, 0, 0, 0, 6, 0,
                                   1, 0, 0, 0, 14, 0, 20, 0, 0, 0, 12, 0, 1, 0, 0, 0, 5, 0, 21, 0, 22, 0, 0],
                    }],
                }],
            fields: vec![Field { access_flags: 18, name_index: 11, descriptor_index: 12, attributes_count: 0, attributes: vec![] },
                         Field { access_flags: 18, name_index: 15, descriptor_index: 16, attributes_count: 0, attributes: vec![] }],
            attributes: vec![Attribute { attribute_name_index: 27, info: vec![0, 28] }],
        }
    }
}