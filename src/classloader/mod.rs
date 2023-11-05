use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use anyhow::Error;

use crate::classloader::io::{find_class, read_bytes, read_f32, read_f64, read_i32, read_i64, read_u16, read_u32, read_u8};
use crate::classloader::classdef::{AttributeType, ClassDef, CpEntry, Exception, Field, Method, MethodCode};

pub(crate) mod classdef;
pub(crate) mod io;

pub(crate) fn get_classdef(classpath: &Vec<String>, class_name: &str) -> Result<ClassDef,Error> {
    println!("read class {} ", class_name);
    let resolved_path = find_class(classpath, class_name)?;
    let bytecode = read_bytecode(resolved_path)?;
    load_class(bytecode)
}

/// reads the binary class file from file path or archive
/// and returns the byte array as Vec
fn read_bytecode(name: String) -> Result<Vec<u8>, Error> {
    let mut buffer;
    if name.contains('#') {
        let parts: Vec<&str> = name.split('#').collect();
        let archive_file = File::open(parts[0])?;
        let mut archive_zip = zip::ZipArchive::new(archive_file)?;
        let mut entry = archive_zip.by_name(parts[1])?;
        buffer = vec![0; entry.size() as usize];
        entry.read_exact(&mut buffer)?;
    } else {
        let mut f = File::open(&name)?;
        let metadata = fs::metadata(&name)?;
        buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer)?;
    }
    Ok(buffer)
}

// The native classoader
fn load_class(bytecode: Vec<u8>) -> Result<ClassDef, Error> {
    let pos = &mut 0;
    check_magic(&bytecode, pos);
    let minor_version = read_u16(&bytecode, pos);
    let major_version = read_u16(&bytecode, pos);

    let constant_pool_count = read_u16(&bytecode, pos);
    let mut constant_pool: HashMap<u16, CpEntry> =
        HashMap::with_capacity(constant_pool_count as usize);
    let mut cp_index = 1;
    while cp_index < constant_pool_count {
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
    let super_class = if super_class != 0 { Some(super_class) } else { None };
    let interfaces_count = read_u16(&bytecode, pos);
    let mut interfaces = vec![];
    for _ in 0..interfaces_count {
        interfaces.push(read_u16(&bytecode, pos));
    }

    let fields_count = read_u16(&bytecode, pos);
    let mut fields = HashMap::new();
    for i in 0..fields_count {
        let field = read_field(constant_pool.clone(), pos, &bytecode, i);
        fields.insert(field.name().to_owned(), field);
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
            panic!("attribute not found"); // bug/not-implemented
        }
    }

    Ok(ClassDef::new(
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
    ))
}

fn check_magic(bytecode: &[u8], pos: &mut usize) {
    if bytecode[*pos..*pos + 4] != [0xCA, 0xFE, 0xBA, 0xBE] {
        panic!("Invalid class file");
    }
    *pos += 4;
}

fn read_constant_pool_entry(cp_index: &mut u16, index: &mut usize, bytecode: &[u8]) -> CpEntry {
    let tag = read_u8(bytecode, index);
    // println!("tag {}", tag);
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
        15 => {
            let reference_kind = read_u8(bytecode, index);
            let reference_index = read_u16(bytecode, index);
            CpEntry::MethodHandle(reference_kind, reference_index)
        }
        16 => {
            let descriptor_index = read_u16(bytecode, index);
            CpEntry::MethodType(descriptor_index)
        }
        // 17 Dynamic,
        18 => {
            let bootstrap_method_attr_index = read_u16(bytecode, index);
            let name_and_type_index = read_u16(bytecode, index);
            CpEntry::InvokeDynamic(bootstrap_method_attr_index, name_and_type_index)
        }
        // 19 Module,
        // 20 Package,
        _ => panic!("cp entry type not recognized"),
    }
}

fn read_field(
    constant_pool: Rc<HashMap<u16, CpEntry>>,
    index: &mut usize,
    bytecode: &[u8],
    field_index: u16,
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
        field_index,
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
                    if let Some(att) = read_attribute(constant_pool.clone(), &info, ci) {
                        code_attributes.insert(att.0, att.1);
                    }
                }
                Some((
                    "Code".into(),
                    AttributeType::Code(Box::new(MethodCode::new(
                        max_stack,
                        max_locals,
                        code,
                        exception_table,
                        code_attributes,
                    ))),
                ))
            }
            "SourceFile" => Some(("SourceFile".into(), AttributeType::SourceFile)), //stub
            "LineNumberTable" => Some(("SourceFile".into(), AttributeType::LineNumberTable)), //stub
            "RuntimeVisibleAnnotations" => {
                Some(("".into(), AttributeType::RuntimeInvisibleAnnotations))
            } //stub
            "NestMembers" => Some(("".into(), AttributeType::NestMembers)),         //stub
            "BootstrapMethods" => Some(("".into(), AttributeType::BootstrapMethods)), //stub
            "InnerClasses" => Some(("".into(), AttributeType::InnerClasses)),       //stub
            "Signature" => Some(("".into(), AttributeType::Signature)),             //stub
            "NestHost" => Some(("".into(), AttributeType::NestHost)),               //stub
            "EnclosingMethod" => Some(("".into(), AttributeType::EnclosingMethod)),               //stub
            "PermittedSubclasses" => Some(("".into(), AttributeType::PermittedSubclasses)),               //stub
            //TODO more actual attribute implementations
            _ => None,
        };
    }
    None
}

