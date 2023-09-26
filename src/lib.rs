pub mod types;
pub mod io;
pub mod opcodes;

use std::collections::HashMap;
use std::rc::Rc;
use crate::io::{read_f32, read_f64, read_i32, read_i64, read_u16, read_u32};
use crate::types::{AttributeType, Class, MethodCode, Exception, Field, Method};

pub fn get_class(bytecode: Vec<u8>) -> Option<Class> {
    check_magic(&bytecode);

    let constant_pool_count = read_u16(&bytecode, 8);
    // println!("cp count: {}", constant_pool_count);
    let mut index = 10;
    let mut constant_pool: HashMap<usize, CpEntry> = HashMap::with_capacity(constant_pool_count as usize);
    let mut cp_index = 1;
    while cp_index < constant_pool_count as usize {
        // println!("cp#{}", cp_index);
        constant_pool.insert(cp_index, read_constant_pool_entry(&mut cp_index, &mut index, &bytecode));
        cp_index += 1;
    }

    let constant_pool = Rc::new(constant_pool);

    let access_flags = read_u16(&bytecode, index);
    let this_class = read_u16(&bytecode, index + 2);
    let super_class = read_u16(&bytecode, index + 4);

    let interfaces_count = read_u16(&bytecode, index + 6);
    // println!("interfaces count: {}", interfaces_count);
    index += 8;
    let mut interfaces = vec![];
    for _ in 0..interfaces_count {
        interfaces.push(read_u16(&bytecode, index));
        index += 2;
    }

    let fields_count = read_u16(&bytecode, index);
    index += 2;
    let mut fields = vec![];
    for _ in 0..fields_count {
        fields.push(read_field(constant_pool.clone(), &mut index, &bytecode));
    }

    let methods_count = read_u16(&bytecode, index);
    index += 2;
    let mut methods = HashMap::new();
    for _ in 0..methods_count {
        let m = read_method(constant_pool.clone(), &mut index, &bytecode);
        methods.insert(m.name(), m);
    }

    let attributes_count = read_u16(&bytecode, index);
    index += 2;
    let mut attributes = HashMap::new();
    for _ in 0..attributes_count {
        let some = read_attribute(constant_pool.clone(), &bytecode, &mut index);
        if let Some(att) = some {
            attributes.insert(att.0, att.1);
        } else {
            panic!(); // bug/not-implemented
        }
    }

    Some(Class {
        minor_version: read_u16(&bytecode, 4),
        major_version: read_u16(&bytecode, 6),
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

fn check_magic(bytecode: &[u8]) {
    if bytecode[0..4] != [0xCA, 0xFE, 0xBA, 0xBE] {
        panic!("Invalid class file");
    }
}

fn read_constant_pool_entry(cp_index: &mut usize, index: &mut usize, bytecode: &[u8]) -> CpEntry {
    let tag = bytecode[*index];
    // println!("#{}: {}", cp_index, tag);
    match tag {
        1 => {
            let len = read_u16(bytecode, *index + 1) as usize;
            let utf: Vec<u8> = Vec::from(&bytecode[*index + 3..*index + 3 + len]);
            *index += len + 3;
            CpEntry::Utf8(String::from_utf8(utf).unwrap())
        }
        3 => {
            let value = read_i32(bytecode, *index + 1);
            *index += 5;
            CpEntry::Integer(value)
        }
        4 => {
            let value = read_f32(bytecode, *index + 1);
            *index += 5;
            CpEntry::Float(value)
        }
        5 => {
            let value = read_i64(bytecode, *index + 1);
            *index += 9;
            let r = CpEntry::Long(value);
            *cp_index += 1;
            r
        }
        6 => {
            let value = read_f64(bytecode, *index + 1);
            *index += 9;
            let r = CpEntry::Double(value);
            *cp_index += 1;
            r
        }
        7 => {
            let name_index = read_u16(bytecode, *index + 1);
            *index += 3;
            CpEntry::ClassRef(name_index)
        }
        8 => {
            let string_index = read_u16(bytecode, *index + 1);
            *index += 3;
            CpEntry::StringRef(string_index)
        }
        9 => {
            let class_index = read_u16(bytecode, *index + 1);
            let name_and_type_index = read_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::Fieldref(class_index, name_and_type_index)
        }
        10 => {
            let class_index = read_u16(bytecode, *index + 1);
            let name_and_type_index = read_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::MethodRef(class_index, name_and_type_index)
        }
        11 => {
            let class_index = read_u16(bytecode, *index + 1);
            let name_and_type_index = read_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::InterfaceMethodref(class_index, name_and_type_index)
        }
        12 => {
            let name_index = read_u16(bytecode, *index + 1) as usize;
            let descriptor_index = read_u16(bytecode, *index + 3) as usize;
            *index += 5;
            CpEntry::NameAndType(name_index, descriptor_index)
        }
        // 15 MethodHandle,
        // 16 MethodType,
        // 17 Dynamic,
        // 18 InvokeDynamic,
        // 19 Module,
        // 20 Package,

        _ => panic!("cp entry type not recognized")
    }
}

fn read_field(constant_pool: Rc<HashMap<usize, CpEntry>>, index: &mut usize, bytecode: &[u8]) -> Field {
    let access_flags = read_u16(bytecode, *index);
    let name_index = read_u16(bytecode, *index + 2) as usize;
    let descriptor_index = read_u16(bytecode, *index + 4) as usize;
    let attributes_count = read_u16(bytecode, *index + 6);
    *index += 8;
    let mut attributes = HashMap::new();
    for _ in 0..attributes_count {
        if let Some(att) = read_attribute(constant_pool.clone(), bytecode, index) {
            attributes.insert(att.0, att.1);
        } else {
            panic!("attribute not recognized"); // bug/not-implemented
        }
    }
    Field::new(
        constant_pool,
        access_flags,
        name_index,
        descriptor_index,
        attributes,
    )
}

fn read_method(constant_pool: Rc<HashMap<usize, CpEntry>>, index: &mut usize, bytecode: &[u8]) -> Method {
    let access_flags = read_u16(bytecode, *index);
    let name_index = read_u16(bytecode, *index + 2) as usize;
    let descriptor_index = read_u16(bytecode, *index + 4) as usize;
    let attributes_count = read_u16(bytecode, *index + 6);
    *index += 8;

    let mut attributes = HashMap::new();
    for _ in 0..attributes_count {
        if let Some(att) = read_attribute(constant_pool.clone(), bytecode, index) {
            attributes.insert(att.0, att.1);
        }
    }

    Method::new(
        constant_pool,
        access_flags,
        name_index,
        descriptor_index,
        attributes,
    )
}

fn read_attribute(constant_pool: Rc<HashMap<usize, CpEntry>>, bytecode: &[u8], index: &mut usize) -> Option<(String, AttributeType)> {
    let attribute_name_index = read_u16(bytecode, *index) as usize;
    *index += 2;
    let attribute_length = read_u32(bytecode, *index) as usize;
    *index += 4;
    let info: Vec<u8> = Vec::from(&bytecode[*index..*index + attribute_length]);
    *index += attribute_length;


    if let CpEntry::Utf8(s) = &constant_pool.get(&attribute_name_index).unwrap() {
        // println!("Att [{}]", s);
        return match s.as_str() {
            "ConstantValue" => {
                assert_eq!(info.len(), 2);
                Some(("ConstantValue".into(), AttributeType::ConstantValue(read_u16(&info, 0))))
            }
            "Code" => {
                let max_stack = read_u16(&info, 0);
                let max_locals = read_u16(&info, 2);
                let code_length = read_u32(&info, 4) as usize;
                let code = Vec::from(&info[8..8 + code_length]);
                let exception_table_length = read_u16(&info, 8 + code_length) as usize;

                let mut code_index = 10 + code_length;
                let mut exception_table = vec![];
                for _ in 0..exception_table_length {
                    exception_table.push(Exception::read(&info, code_index));
                    code_index += 8;
                }
                let attribute_count = read_u16(&info, code_index);
                code_index += 2;
                let mut code_attributes = HashMap::new();
                for _ in 0..attribute_count {
                    if let Some(att) = read_attribute(constant_pool.clone(), &info, &mut code_index) {
                        code_attributes.insert(att.0, att.1);
                    }
                }
                Some(("Code".into(), AttributeType::Code(MethodCode::new(max_stack, max_locals, code, exception_table, code_attributes))))
            }
            "SourceFile" => Some(("SourceFile".into(), AttributeType::SourceFile)),
            "LineNumberTable" => Some(("SourceFile".into(), AttributeType::LineNumberTable)),
            _ => None
        };
    }
    None
}

#[derive(Debug)]
pub enum CpEntry {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    ClassRef(u16),
    StringRef(u16),
    Fieldref(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodref(u16, u16),
    NameAndType(usize, usize),
}

