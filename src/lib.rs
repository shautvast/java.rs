pub mod types;
mod io;

use std::rc::Rc;
use crate::io::{read_f32, read_f64, read_i32, read_i64, read_u16, read_u32};
use crate::types::{AttributeType, Class, MethodCode, Exception, Field, Method};

pub fn get_class(bytecode: Vec<u8>) -> Option<Class> {
    check_magic(&bytecode);

    let constant_pool_count = read_u16(&bytecode, 8);
    let mut index = 10;
    let mut constant_pool: Vec<CpEntry> = vec![];
    for cp_index in 0..constant_pool_count - 1 {
        constant_pool.push(read_constant_pool_entry((cp_index + 1) as usize, &mut index, &bytecode));
    }

    let constant_pool = Rc::new(constant_pool);

    let access_flags = read_u16(&bytecode, index);
    let this_class = read_u16(&bytecode, index + 2);
    let super_class = read_u16(&bytecode, index + 4);

    let interfaces_count = read_u16(&bytecode, index + 6);
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
    let mut methods = vec![];
    for _ in 0..methods_count {
        methods.push(read_method(constant_pool.clone(), &mut index, &bytecode));
    }

    let attributes_count = read_u16(&bytecode, index);
    index += 2;
    let mut attributes = vec![];
    for _ in 0..attributes_count {
        let some = read_attribute(constant_pool.clone(), &bytecode, &mut index);
        if let Some(att) = some {
            attributes.push(att);
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

fn read_constant_pool_entry(cp_index: usize, index: &mut usize, bytecode: &[u8]) -> CpEntry {
    let tag = bytecode[*index];
    match tag {
        1 => {
            let len = read_u16(bytecode, *index + 1) as usize;
            let utf: Vec<u8> = Vec::from(&bytecode[*index + 3..*index + 3 + len]);
            *index += len + 3;
            CpEntry::Utf8(cp_index, String::from_utf8(utf).unwrap())
        }
        3 => {
            let value = read_i32(bytecode, *index + 1);
            *index += 5;
            CpEntry::Integer(cp_index, value)
        }
        4 => {
            let value = read_f32(bytecode, *index + 1);
            *index += 5;
            CpEntry::Float(cp_index, value)
        }
        5 => {
            let value = read_i64(bytecode, *index + 1);
            *index += 9;
            CpEntry::Long(cp_index, value)
        }
        6 => {
            let value = read_f64(bytecode, *index + 1);
            *index += 9;
            CpEntry::Double(cp_index, value)
        }
        7 => {
            let name_index = read_u16(bytecode, *index + 1);
            *index += 3;
            CpEntry::ClassRef(cp_index, name_index)
        }
        8 => {
            let string_index = read_u16(bytecode, *index + 1);
            *index += 3;
            CpEntry::StringRef(cp_index, string_index)
        }
        9 => {
            let class_index = read_u16(bytecode, *index + 1);
            let name_and_type_index = read_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::Fieldref(cp_index, class_index, name_and_type_index)
        }
        10 => {
            let class_index = read_u16(bytecode, *index + 1);
            let name_and_type_index = read_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::MethodRef(cp_index, class_index, name_and_type_index)
        }
        11 => {
            let class_index = read_u16(bytecode, *index + 1);
            let name_and_type_index = read_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::InterfaceMethodref(cp_index, class_index, name_and_type_index)
        }
        12 => {
            let name_index = read_u16(bytecode, *index + 1) as usize;
            let descriptor_index = read_u16(bytecode, *index + 3) as usize;
            *index += 5;
            CpEntry::NameAndType(cp_index, name_index, descriptor_index)
        }
        // 15 MethodHandle,
        // 16 MethodType,
        // 17 Dynamic,
        // 18 InvokeDynamic,
        // 19 Module,
        // 20 Package,
        _ => panic!()
    }
}

fn read_field(constant_pool: Rc<Vec<CpEntry>>, index: &mut usize, bytecode: &[u8]) -> Field {
    let access_flags = read_u16(bytecode, *index);
    let name_index = read_u16(bytecode, *index + 2) as usize;
    let descriptor_index = read_u16(bytecode, *index + 4) as usize;
    let attributes_count = read_u16(bytecode, *index + 6);
    *index += 8;
    let mut attributes = vec![];
    for _ in 0..attributes_count {
        if let Some(att) = read_attribute(constant_pool.clone(), bytecode, index) {
            attributes.push(att);
        } else {
            panic!(); // bug/not-implemented
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

fn read_method(constant_pool: Rc<Vec<CpEntry>>, index: &mut usize, bytecode: &[u8]) -> Method {
    let access_flags = read_u16(bytecode, *index);
    let name_index = read_u16(bytecode, *index + 2) as usize;
    let descriptor_index = read_u16(bytecode, *index + 4) as usize;
    let attributes_count = read_u16(bytecode, *index + 6);
    *index += 8;

    let mut attributes = vec![];
    for _ in 0..attributes_count {
        if let Some(att) = read_attribute(constant_pool.clone(), bytecode, index) {
            attributes.push(att);
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

fn read_attribute(constant_pool: Rc<Vec<CpEntry>>, bytecode: &[u8], index: &mut usize) -> Option<AttributeType> {
    let attribute_name_index = read_u16(bytecode, *index) as usize;
    *index += 2;
    let attribute_length = read_u32(bytecode, *index) as usize;
    *index += 4;
    let info: Vec<u8> = Vec::from(&bytecode[*index..*index + attribute_length]);
    *index += attribute_length;


    if let CpEntry::Utf8(_, s) = &constant_pool[attribute_name_index - 1] {
        // println!("{}", s);
        return match s.as_str() {
            "ConstantValue" => {
                assert_eq!(info.len(), 2);
                Some(AttributeType::ConstantValue(read_u16(&info, 0)))
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
                let mut code_attributes = vec![];
                for _ in 0..attribute_count {
                    if let Some(att) = read_attribute(constant_pool.clone(), &info, &mut code_index) {
                        code_attributes.push(att);
                    }
                }
                Some(AttributeType::Code(MethodCode::new(max_stack, max_locals, code, exception_table, code_attributes)))
            }
            "SourceFile" => Some(AttributeType::SourceFile),
            _ => None
        };
    }
    None
}

#[derive(Debug)]
pub enum CpEntry {
    Utf8(usize, String),
    Integer(usize, i32),
    Float(usize, f32),
    Long(usize, i64),
    Double(usize, f64),
    ClassRef(usize, u16),
    StringRef(usize, u16),
    Fieldref(usize, u16, u16),
    MethodRef(usize, u16, u16),
    InterfaceMethodref(usize, u16, u16),
    NameAndType(usize, usize, usize),
}

