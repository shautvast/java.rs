pub mod types;

use std::rc::Rc;
use crate::types::{CpEntry, Class, Field, Attribute, Method};

pub fn get_class(bytecode: Vec<u8>) -> Option<Class> {
    check_magic(&bytecode);

    let constant_pool_count = get_u16(&bytecode, 8);
    let mut index = 10;
    let mut constant_pool: Vec<CpEntry> = vec![];
    for _ in 0..constant_pool_count - 1 {
        constant_pool.push(read_constant_pool_entry(&mut index, &bytecode));
    }

    let constant_pool = Rc::new(constant_pool);

    let access_flags = get_u16(&bytecode, index);
    let this_class = get_u16(&bytecode, index + 2);
    let super_class = get_u16(&bytecode, index + 4);

    let interfaces_count = get_u16(&bytecode, index + 6);
    index += 8;
    let mut interfaces = vec![];
    for _ in 0..interfaces_count {
        interfaces.push(get_u16(&bytecode, index));
        index += 2;
    }

    let fields_count = get_u16(&bytecode, index);
    index += 2;
    let mut fields = vec![];
    for _ in 0..fields_count {
        fields.push(read_field(&mut index, &bytecode));
    }

    let methods_count = get_u16(&bytecode, index);
    index += 2;
    let mut methods = vec![];
    for _ in 0..methods_count {
        methods.push(read_method(constant_pool.clone(), &mut index, &bytecode));
    }

    let attributes_count = get_u16(&bytecode, index);
    index += 2;
    let mut attributes = vec![];
    for _ in 0..attributes_count {
        attributes.push(read_attribute(&bytecode, &mut index));
    }

    Some(Class {
        minor_version: get_u16(&bytecode, 4),
        major_version: get_u16(&bytecode, 6),
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

fn check_magic(bytecode: &Vec<u8>) {
    if &bytecode[0..4] != [0xCA, 0xFE, 0xBA, 0xBE] {
        panic!("Invalid class file");
    }
}

fn read_constant_pool_entry(index: &mut usize, bytecode: &Vec<u8>) -> CpEntry {
    let tag = bytecode[*index];
    match tag {
        1 => {
            let len = get_u16(bytecode, *index + 1) as usize;
            let utf: Vec<u8> = Vec::from(&bytecode[*index + 3..*index + 3 + len]);
            *index += len + 3;
            CpEntry::Utf8(String::from_utf8(utf).unwrap())
        }
        3 => {
            let value = get_i32(bytecode, *index + 1);
            *index += 5;
            CpEntry::Integer(value)
        }
        4 => {
            let value = get_f32(bytecode, *index + 1);
            *index += 5;
            CpEntry::Float(value)
        }
        5 => {
            let value = get_i64(bytecode, *index + 1);
            *index += 9;
            CpEntry::Long(value)
        }
        6 => {
            let value = get_f64(bytecode, *index + 1);
            *index += 9;
            CpEntry::Double(value)
        }
        7 => {
            let name_index = get_u16(bytecode, *index + 1);
            *index += 3;
            CpEntry::ClassRef(name_index)
        }
        8 => {
            let string_index = get_u16(bytecode, *index + 1);
            *index += 3;
            CpEntry::StringRef(string_index)
        }
        9 => {
            let class_index = get_u16(bytecode, *index + 1);
            let name_and_type_index = get_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::Fieldref(class_index, name_and_type_index)
        }
        10 => {
            let class_index = get_u16(bytecode, *index + 1);
            let name_and_type_index = get_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::MethodRef(class_index, name_and_type_index)
        }
        11 => {
            let class_index = get_u16(bytecode, *index + 1);
            let name_and_type_index = get_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::InterfaceMethodref(class_index, name_and_type_index)
        }
        12 => {
            let name_index = get_u16(bytecode, *index + 1);
            let descriptor_index = get_u16(bytecode, *index + 3);
            *index += 5;
            CpEntry::NameAndType(name_index, descriptor_index)
        },
        // 15 MethodHandle,
        // 16 MethodType,
        // 17 Dynamic,
        // 18 InvokeDynamic,
        // 19 Module,
        // 20 Package,
        _ => panic!()
    }
}

fn read_field(index: &mut usize, bytecode: &Vec<u8>) -> Field {
    let access_flags = get_u16(bytecode, *index);
    let name_index = get_u16(bytecode, *index + 2);
    let descriptor_index = get_u16(bytecode, *index + 4);
    let attributes_count = get_u16(bytecode, *index + 6);
    *index += 8;
    let mut attributes = vec![];
    for _ in 0..attributes_count {
        attributes.push(read_attribute(bytecode, index));
    }
    Field {
        access_flags,
        name_index,
        descriptor_index,
        attributes_count,
        attributes,
    }
}

fn read_method(constant_pool: Rc<Vec<CpEntry>>, index: &mut usize, bytecode: &Vec<u8>) -> Method {
    let access_flags = get_u16(bytecode, *index);
    let name_index = get_u16(bytecode, *index + 2) as usize;
    let descriptor_index = get_u16(bytecode, *index + 4);
    let attributes_count = get_u16(bytecode, *index + 6);
    *index += 8;
    let mut attributes = vec![];
    for _ in 0..attributes_count {
        attributes.push(read_attribute(bytecode, index));
    }
    Method {
        constant_pool,
        access_flags,
        name_index,
        descriptor_index,
        attributes_count,
        attributes,
    }
}

fn read_attribute(bytecode: &Vec<u8>, index: &mut usize) -> Attribute {
    let attribute_name_index = get_u16(bytecode, *index);
    *index += 2;
    let attribute_length = read_u32(bytecode, *index) as usize;
    *index += 4;
    let info: Vec<u8> = Vec::from(&bytecode[*index..*index + attribute_length]);
    *index += attribute_length;

    Attribute {
        attribute_name_index,
        info,
    }
}

fn get_u16(data: &Vec<u8>, pos: usize) -> u16 {
    u16::from_be_bytes(data[pos..pos + 2].try_into().expect("slice with incorrect length"))
}

fn get_i32(data: &Vec<u8>, pos: usize) -> i32 {
    i32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

fn read_u32(data: &Vec<u8>, pos: usize) -> u32 {
    u32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

fn get_f32(data: &Vec<u8>, pos: usize) -> f32 {
    f32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

fn get_i64(data: &Vec<u8>, pos: usize) -> i64 {
    i64::from_be_bytes(data[pos..pos + 8].try_into().expect("slice with incorrect length"))
}

fn get_f64(data: &Vec<u8>, pos: usize) -> f64 {
    f64::from_be_bytes(data[pos..pos + 8].try_into().expect("slice with incorrect length"))
}