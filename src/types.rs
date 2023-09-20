use std::borrow::ToOwned;
use std::rc::Rc;
use crate::types::CpEntry::*;

#[derive(Debug)]
//TODO create factory function
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Rc<Vec<CpEntry>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

impl<'a> Class {
    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    pub fn get_methods(&self) -> &Vec<Method> {
        &self.methods
    }
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

#[derive(Debug)]
pub struct Field {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub struct Attribute {
    pub attribute_name_index: u16,
    pub info: Vec<u8>,
}

#[derive(Debug)]
pub struct Method {
    pub constant_pool: Rc<Vec<CpEntry>>,
    pub access_flags: u16,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub attributes_count: u16,
    pub attributes: Vec<Attribute>,
}

impl Method {
    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);
        if let Utf8(s) = &self.constant_pool[&self.name_index - 1] {
            full_name.push_str(s);
        }
        if let Utf8(s) = &self.constant_pool[&self.descriptor_index - 1] {
            full_name.push_str(s);
        }


        full_name
    }
}

const MODIFIERS: [(u16, &'static str); 12] = [
    (0x0001, "public "),
    (0x0002, "private "),
    (0x0004, "protected "),
    (0x0008, "static "),
    (0x0010, "final "),
    (0x0020, "synchronized "),
    (0x0040, "volatile "),
    (0x0080, "transient "),
    (0x0100, "native "),
    (0x0200, "interface "),
    (0x0400, "interface "),
    (0x0800, "strict ")];

pub fn get_modifier (value: u16) -> String {
    let mut output = String::new();
    for m in MODIFIERS {
        if value & m.0 == m.0 { output.push_str(&m.1) }
    }
    output
}
