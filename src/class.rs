use std::cell::{RefCell, UnsafeCell};
use crate::classloader::CpEntry;
use crate::heap::Object;
use anyhow::{anyhow, Error};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::io::read_u16;

#[derive(Debug)]
//TODO create factory function
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Rc<HashMap<u16, CpEntry>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: HashMap<String, Method>,
    pub attributes: HashMap<String, AttributeType>,
}

impl Class {
    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    pub fn get_method(&self, name: &str) -> Result<&Method, Error> {
        self.methods
            .get(name)
            .ok_or(anyhow!("Method {} not found", name))
    }

    pub fn get_name(&self) -> &str {
        if let CpEntry::ClassRef(name_index ) = self.constant_pool.get(&self.this_class).unwrap(){
            if let CpEntry::Utf8(name) = self.constant_pool.get(name_index).unwrap(){
                return name;
            }
        }
        panic!();
    }
}
unsafe impl Send for Class {}
unsafe impl Sync for Class {}

pub struct Method {
    pub(crate) constant_pool: Rc<HashMap<u16, CpEntry>>,
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    pub(crate) attributes: HashMap<String, AttributeType>,
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Method {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
            self.access_flags, self.name_index, self.descriptor_index, self.attributes
        )
    }
}

impl Method {
    pub fn new(
        constant_pool: Rc<HashMap<u16, CpEntry>>,
        access_flags: u16,
        name_index: u16,
        descriptor_index: u16,
        attributes: HashMap<String, AttributeType>,
    ) -> Self {
        Method {
            constant_pool,
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        }
    }

    pub fn name(&self) -> String {
        let mut full_name = String::new();
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.name_index).unwrap() {
            full_name.push_str(s);
        }
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.descriptor_index).unwrap() {
            full_name.push_str(s);
        }

        full_name
    }
}

pub struct Field {
    constant_pool: Rc<HashMap<u16, CpEntry>>,
    access_flags: u16,
    pub(crate) name_index: u16,
    descriptor_index: u16,
    attributes: HashMap<String, AttributeType>,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Field {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
            self.access_flags, self.name_index, self.descriptor_index, self.attributes
        )
    }
}

impl Field {
    pub fn new(
        constant_pool: Rc<HashMap<u16, CpEntry>>,
        access_flags: u16,
        name_index: u16,
        descriptor_index: u16,
        attributes: HashMap<String, AttributeType>,
    ) -> Self {
        Field {
            constant_pool,
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        }
    }

    pub fn name(&self) -> String {
        let mut name = String::new();

        name.push(' ');
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.name_index).unwrap() {
            name.push_str(s);
        }

        name
    }

    pub fn type_of(&self) -> &String {
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.descriptor_index).unwrap() {
            return s;
        }
        panic!()
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
    (0x0800, "strict "),
];

pub fn get_modifier(modifier: u16) -> String {
    let mut output = String::new();
    for m in MODIFIERS {
        if modifier & m.0 == m.0 {
            output.push_str(m.1)
        }
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
    pub fn read(code: &[u8], index: &mut usize) -> Self {
        Self {
            start_pc: read_u16(code, index),
            end_pc: read_u16(code, index),
            handler_pc: read_u16(code, index),
            catch_type: read_u16(code, index),
        }
    }
}

#[derive(Debug)]
pub struct MethodCode {
    _max_stack: u16,
    _max_locals: u16,
    pub(crate) opcodes: Vec<u8>,
    _exception_table: Vec<Exception>,
    _code_attributes: HashMap<String, AttributeType>,
}

impl MethodCode {
    pub(crate) fn new(
        _max_stack: u16,
        _max_locals: u16,
        code: Vec<u8>,
        _exception_table: Vec<Exception>,
        _code_attributes: HashMap<String, AttributeType>,
    ) -> Self {
        Self {
            _max_stack,
            _max_locals,
            opcodes: code,
            _exception_table,
            _code_attributes,
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Void,
    Null,
    // the $1_000_000 mistake
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BOOL(bool),
    CHAR(char),
    Ref(Arc<UnsafeCell<Object>>),
}

unsafe impl Send for Value {}
unsafe impl Sync for Value {}
