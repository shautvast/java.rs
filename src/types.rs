use std::rc::Rc;
use crate::CpEntry;

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
    pub attributes: Vec<AttributeType>,
}

impl Class {
    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }
}

pub struct Method {
    constant_pool: Rc<Vec<CpEntry>>,
    access_flags: u16,
    name_index: usize,
    descriptor_index: usize,
    attributes: Vec<AttributeType>,
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Method {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
               self.access_flags, self.name_index, self.descriptor_index, self.attributes)
    }
}

impl Method {
    pub fn new(constant_pool: Rc<Vec<CpEntry>>,
               access_flags: u16,
               name_index: usize,
               descriptor_index: usize,
               attributes: Vec<AttributeType>, ) -> Self {
        Method { constant_pool, access_flags, name_index, descriptor_index, attributes }
    }

    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);
        if let CpEntry::Utf8(_, s) = &self.constant_pool[&self.name_index - 1] {
            full_name.push_str(s);
        }
        if let CpEntry::Utf8(_, s) = &self.constant_pool[&self.descriptor_index - 1] {
            full_name.push_str(s);
        }


        full_name
    }

    // pub fn get_code(&self) {
    //     for att in &self.attributes {
    //         if let CpEntry::Utf8(_, str) = &self.constant_pool[&att.attribute_name_index - 1] {
    //             println!("{}", str);
    //             if str == "Code" {
    //                 println!("{:?}", att.info);
    //             }
    //         }
    //     }
    // }
}

pub struct Field {
    constant_pool: Rc<Vec<CpEntry>>,
    access_flags: u16,
    name_index: usize,
    descriptor_index: usize,
    attributes: Vec<AttributeType>,
}

use std::fmt;
use crate::io::read_u16;

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
               self.access_flags, self.name_index, self.descriptor_index, self.attributes)
    }
}

impl Field {
    pub fn new(constant_pool: Rc<Vec<CpEntry>>,
               access_flags: u16,
               name_index: usize,
               descriptor_index: usize,
               attributes: Vec<AttributeType>, ) -> Self {
        Field { constant_pool, access_flags, name_index, descriptor_index, attributes: attributes }
    }

    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);

        if let CpEntry::Utf8(_, s) = &self.constant_pool[&self.descriptor_index - 1] {
            full_name.push_str(s);
        }
        full_name.push(' ');
        if let CpEntry::Utf8(_, s) = &self.constant_pool[&self.name_index - 1] {
            full_name.push_str(s);
        }

        full_name
    }
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

pub fn get_modifier(modifier: u16) -> String {
    let mut output = String::new();
    for m in MODIFIERS {
        if modifier & m.0 == m.0 { output.push_str(m.1) }
    }
    output
}


#[derive(Debug)]
pub enum AttributeType {
    ConstantValue(u16),
    Code(MethodCode),
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

#[derive(Debug)]
pub struct Exception {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

impl Exception {
    pub fn read(code: &[u8], index: usize) -> Self {
        Self {
            start_pc: read_u16(code, index),
            end_pc: read_u16(code, index + 2),
            handler_pc: read_u16(code, index + 4),
            catch_type: read_u16(code, index + 6),
        }
    }
}

#[derive(Debug)]
pub struct MethodCode {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_table: Vec<Exception>,
    code_attributes: Vec<AttributeType>,
}

impl MethodCode {
    pub(crate) fn new(max_stack: u16, max_locals: u16,
                      code: Vec<u8>,
                      exception_table: Vec<Exception>,
                      code_attributes: Vec<AttributeType>) -> Self {
        Self { max_stack, max_locals, code, exception_table, code_attributes }
    }
}