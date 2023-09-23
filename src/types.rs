use std::rc::Rc;
use crate::CpEntry;
use crate::CpEntry::NameAndType;

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

impl Class {
    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }
}

#[derive(Debug)]
pub struct Method {
    constant_pool: Rc<Vec<CpEntry>>,
    access_flags: u16,
    name_index: usize,
    descriptor_index: usize,
    attributes: Vec<Attribute>,
}

impl Method {
    pub fn new(constant_pool: Rc<Vec<CpEntry>>,
               access_flags: u16,
               name_index: usize,
               descriptor_index: usize,
               attributes: Vec<Attribute>, ) -> Self {
        Method { constant_pool, access_flags, name_index, descriptor_index, attributes }
    }

    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);
        if let CpEntry::Utf8(s) = &self.constant_pool[&self.name_index - 1] {
            full_name.push_str(s);
        }
        if let CpEntry::Utf8(s) = &self.constant_pool[&self.descriptor_index - 1] {
            full_name.push_str(s);
        }


        full_name
    }

    pub fn get_code(&self) {
        for att in &self.attributes {
            if let CpEntry::Utf8(str) = &self.constant_pool[&att.attribute_name_index - 1] {
                println!("{}", str);
                if str == "Code" {
                    println!("{:?}", att.info);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Field {
    constant_pool: Rc<Vec<CpEntry>>,
    access_flags: u16,
    name_index: usize,
    descriptor_index: usize,
    _attributes: Vec<Attribute>,
}

impl Field {
    pub fn new(constant_pool: Rc<Vec<CpEntry>>,
               access_flags: u16,
               name_index: usize,
               descriptor_index: usize,
               attributes: Vec<Attribute>, ) -> Self {
        Field { constant_pool, access_flags, name_index, descriptor_index, _attributes: attributes }
    }

    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);

        if let CpEntry::Utf8(s) = &self.constant_pool[&self.descriptor_index - 1] {
            full_name.push_str(s);
        }
        full_name.push(' ');
        if let CpEntry::Utf8(s) = &self.constant_pool[&self.name_index - 1] {
            full_name.push_str(s);
        }

        full_name
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub attribute_name_index: usize,
    pub info: Vec<u8>,
}


const MODIFIERS: [(u16, &str); 12] = [
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

pub fn get_modifier(value: u16) -> String {
    let mut output = String::new();
    for m in MODIFIERS {
        if value & m.0 == m.0 { output.push_str(m.1) }
    }
    output
}

use std::cell::OnceCell;
use std::collections::HashMap;
use crate::types::AttributeType::{BootstrapMethods, Code, ConstantValue, NestHost, NestMembers, PermittedSubclasses, StackMapTable};

enum AttributeType {
    ConstantValue,
    Code,
    StackMapTable,
    BootstrapMethods,
    NestHost,
    NestMembers,
    PermittedSubclasses,
    Exceptions,
    InnerClasses,
    EnclosingMethod,
    Synthetic,
    Signature,
    Record,
    SourceFile,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    SourceDebugExtension,
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault,
    MethodParameters,
    Module,
    ModulePackages,
    ModuleMainClass,
}

const cell: OnceCell<HashMap<&str, AttributeType>> = OnceCell::new();

const value: &HashMap<&str, AttributeType> = cell.get_or_init(|| {
    let mut map = HashMap::with_capacity(18);
    map.insert("ConstantValue", ConstantValue);
    map.insert("Code", Code);
    map.insert("StackMapTable", StackMapTable);
    map.insert("BootstrapMethods", BootstrapMethods);
    map.insert("NestHost", NestHost);
    map.insert("NestMembers", NestMembers);
    map.insert("PermittedSubclasses", PermittedSubclasses);
    map
});