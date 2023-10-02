use crate::class::{AttributeType, Class, Exception, Field, Method, MethodCode};
use crate::io::{read_bytes, read_f32, read_f64, read_i32, read_i64, read_u16, read_u32, read_u8};
use anyhow::Error;
use std::collections::HashMap;
use std::rc::Rc;

pub fn load_class(bytecode: Vec<u8>) -> Result<Class, Error> {
    let pos = &mut 0;
    check_magic(&bytecode, pos);
    let minor_version = read_u16(&bytecode, pos);
    let major_version = read_u16(&bytecode, pos);

    let constant_pool_count = read_u16(&bytecode, pos);
    // println!("cp count: {}", constant_pool_count);
    let mut constant_pool: HashMap<u16, CpEntry> =
        HashMap::with_capacity(constant_pool_count as usize);
    let mut cp_index = 1;
    while cp_index < constant_pool_count {
        // println!("cp#{}", cp_index);
        constant_pool.insert(
            cp_index,
            read_constant_pool_entry(&mut cp_index, pos, &bytecode),
        );
        cp_index += 1;
    }

    let constant_pool = Rc::new(constant_pool);

    let access_flags = read_u16(&bytecode, pos);
    let this_class = read_u16(&bytecode, pos);
    let super_class = read_u16(&bytecode, pos);

    let interfaces_count = read_u16(&bytecode, pos);
    let mut interfaces = vec![];
    for _ in 0..interfaces_count {
        interfaces.push(read_u16(&bytecode, pos));
    }

    let fields_count = read_u16(&bytecode, pos);
    let mut fields = vec![];
    for _ in 0..fields_count {
        fields.push(read_field(constant_pool.clone(), pos, &bytecode));
    }

    let methods_count = read_u16(&bytecode, pos);
    let mut methods = HashMap::new();
    for _ in 0..methods_count {
        let m = read_method(constant_pool.clone(), pos, &bytecode);
        methods.insert(m.name(), m);
    }

    let attributes_count = read_u16(&bytecode, pos);
    let mut attributes = HashMap::new();
    for _ in 0..attributes_count {
        let some = read_attribute(constant_pool.clone(), &bytecode, pos);
        if let Some(att) = some {
            attributes.insert(att.0, att.1);
        } else {
            panic!(); // bug/not-implemented
        }
    }

    Ok(Class {
        minor_version,
        major_version,
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

fn check_magic(bytecode: &[u8], pos: &mut usize) {
    if bytecode[*pos..*pos + 4] != [0xCA, 0xFE, 0xBA, 0xBE] {
        panic!("Invalid class file");
    }
    *pos += 4;
}

fn read_constant_pool_entry(cp_index: &mut u16, index: &mut usize, bytecode: &[u8]) -> CpEntry {
    let tag = read_u8(bytecode, index);
    match tag {
        1 => {
            let len = read_u16(bytecode, index) as usize;
            let utf: Vec<u8> = read_bytes(&bytecode, index, len);
            CpEntry::Utf8(String::from_utf8(utf).unwrap())
        }
        3 => {
            let value = read_i32(bytecode, index);
            CpEntry::Integer(value)
        }
        4 => {
            let value = read_f32(bytecode, index);
            CpEntry::Float(value)
        }
        5 => {
            let value = read_i64(bytecode, index);
            let val = CpEntry::Long(value);
            *cp_index += 1;
            val
        }
        6 => {
            let value = read_f64(bytecode, index);
            let val = CpEntry::Double(value); //TODO order can be smarter
            *cp_index += 1;
            val
        }
        7 => {
            let name_index = read_u16(bytecode, index);
            CpEntry::ClassRef(name_index)
        }
        8 => {
            let string_index = read_u16(bytecode, index);
            CpEntry::StringRef(string_index)
        }
        9 => {
            let class_index = read_u16(bytecode, index);
            let name_and_type_index = read_u16(bytecode, index);
            CpEntry::Fieldref(class_index, name_and_type_index)
        }
        10 => {
            let class_index = read_u16(bytecode, index);
            let name_and_type_index = read_u16(bytecode, index);
            CpEntry::MethodRef(class_index, name_and_type_index)
        }
        11 => {
            let class_index = read_u16(bytecode, index);
            let name_and_type_index = read_u16(bytecode, index);
            CpEntry::InterfaceMethodref(class_index, name_and_type_index)
        }
        12 => {
            let name_index = read_u16(bytecode, index);
            let descriptor_index = read_u16(bytecode, index);
            CpEntry::NameAndType(name_index, descriptor_index)
        }
        // 15 MethodHandle,
        // 16 MethodType,
        // 17 Dynamic,
        // 18 InvokeDynamic,
        // 19 Module,
        // 20 Package,
        _ => panic!("cp entry type not recognized"),
    }
}

fn read_field(
    constant_pool: Rc<HashMap<u16, CpEntry>>,
    index: &mut usize,
    bytecode: &[u8],
) -> Field {
    let access_flags = read_u16(bytecode, index);
    let name_index = read_u16(bytecode, index);
    let descriptor_index = read_u16(bytecode, index);
    let attributes_count = read_u16(bytecode, index);
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

fn read_method(
    constant_pool: Rc<HashMap<u16, CpEntry>>,
    index: &mut usize,
    bytecode: &[u8],
) -> Method {
    let access_flags = read_u16(bytecode, index);
    let name_index = read_u16(bytecode, index);
    let descriptor_index = read_u16(bytecode, index);
    let attributes_count = read_u16(bytecode, index);

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

fn read_attribute(
    constant_pool: Rc<HashMap<u16, CpEntry>>,
    bytecode: &[u8],
    index: &mut usize,
) -> Option<(String, AttributeType)> {
    let attribute_name_index = read_u16(bytecode, index);
    let attribute_length = read_u32(bytecode, index) as usize;
    let info: Vec<u8> = Vec::from(&bytecode[*index..*index + attribute_length]);
    *index += attribute_length;

    if let CpEntry::Utf8(s) = &constant_pool.get(&attribute_name_index).unwrap() {
        // println!("Att [{}]", s);
        return match s.as_str() {
            "ConstantValue" => {
                assert_eq!(info.len(), 2);
                Some((
                    "ConstantValue".into(),
                    AttributeType::ConstantValue(read_u16(&info, &mut 0)),
                ))
            }
            "Code" => {
                let ci = &mut 0;
                let max_stack = read_u16(&info, ci);
                let max_locals = read_u16(&info, ci);
                let code_length = read_u32(&info, ci) as usize;
                let code = read_bytes(&info, ci, code_length);
                let exception_table_length = read_u16(&info, ci) as usize;

                let mut exception_table = vec![];
                for _ in 0..exception_table_length {
                    exception_table.push(Exception::read(&info, ci));
                }
                let attribute_count = read_u16(&info, ci);
                let mut code_attributes = HashMap::new();
                for _ in 0..attribute_count {
                    if let Some(att) = read_attribute(constant_pool.clone(), &info, ci)
                    {
                        code_attributes.insert(att.0, att.1);
                    }
                }
                Some((
                    "Code".into(),
                    AttributeType::Code(MethodCode::new(
                        max_stack,
                        max_locals,
                        code,
                        exception_table,
                        code_attributes,
                    )),
                ))
            }
            "SourceFile" => Some(("SourceFile".into(), AttributeType::SourceFile)),
            "LineNumberTable" => Some(("SourceFile".into(), AttributeType::LineNumberTable)),
            _ => None,
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
    NameAndType(u16, u16),
}
