use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Error};

use crate::classloader::CpEntry;
use crate::heap::ObjectRef;
use crate::io::read_u16;

/// the class definition as read from the class file + derived values
// TODO implement call to static initializers
// TODO implement storage for static fields
#[derive(Debug)]
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Rc<HashMap<u16, CpEntry>>,
    pub access_flags: u16,
    pub name: String,
    pub super_class_name: Option<String>,
    pub super_class: Option<Rc<Class>>,
    pub interface_indices: Vec<u16>,
    pub interfaces: Vec<Class>,
    pub fields: HashMap<String, Field>,
    pub methods: HashMap<String, Method>,
    pub attributes: HashMap<String, AttributeType>,
    pub(crate) field_mapping: Option<HashMap<String, HashMap<String, (String, usize)>>>, // first key: this/super/supersuper-name(etc), second key: fieldname, value (type, index). See below
}

impl Class {
    pub fn new(minor_version: u16,
               major_version: u16,
               constant_pool: Rc<HashMap<u16, CpEntry>>,
               access_flags: u16,
               this_class: u16,
               super_class_index: u16,
               interface_indices: Vec<u16>,
               fields: HashMap<String, Field>,
               methods: HashMap<String, Method>,
               attributes: HashMap<String, AttributeType>) -> Self {
        let name = Class::class_name(this_class, constant_pool.clone()).unwrap();
        let super_class_name = Class::class_name(super_class_index, constant_pool.clone());

        Self {
            major_version,
            minor_version,
            constant_pool,
            access_flags,
            name,
            super_class_name,
            super_class: None, // has to be instantiated later, because it involves classloading. maybe not store it here
            interface_indices,
            interfaces: vec![], // same
            fields,
            methods,
            attributes,
            field_mapping: None,
        }
    }

    pub(crate) fn n_fields(&self) -> usize {
        self.field_mapping.as_ref().map_or(0, |m| m.len())
    }

    // Create a mapping per field(name) to an index in the storage vector that contains the instance data.
    // When a field is stored, first the index will be looked up, using the qualified name (from the FieldRef)
    // The qualified name is the combination of class name and field name.
    // The class name is needed as part of the key to separate class from superclass fields
    // (duplicates in the singular field name are allowed).
    // This way `this.a` can be differentiated from `super.a`.
    //
    // this method looks up this and super classes and calls map_fields for each.
    pub fn initialize(&mut self) {
        let mut field_mapping = HashMap::new();
        let mut field_map_index: usize = 0;

        Class::map_fields(&mut field_mapping, self, &mut field_map_index);
        let mut sooper = &self.super_class;
        while let Some(super_class) = sooper {
            Class::map_fields(&mut field_mapping, super_class, &mut field_map_index);
            sooper = &super_class.super_class;
        }
        self.field_mapping = Some(field_mapping);
    }

    // part of the initialize procedure
    fn map_fields(field_mapping: &mut HashMap<String, HashMap<String, (String, usize)>>, class: &Class, field_map_index: &mut usize) {
        let mut this_fields = HashMap::new(); //fields in class are stored per class and every superclass.
        for field in &class.fields {
            this_fields.insert(field.0.to_owned(), (field.1.type_of().to_owned(), *field_map_index)); //name => (type,index)
            *field_map_index += 1;
        }
        let this_name = class.name.to_owned();
        field_mapping.insert(this_name, this_fields);
    }

    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    pub fn get_method(&self, name: &str) -> Result<&Method, Error> {
        self.methods
            .get(name)
            .ok_or(anyhow!("Method {} not found", name))
    }

    fn class_name(super_class_index: u16, constant_pool: Rc<HashMap<u16, CpEntry>>) -> Option<String> {
        if super_class_index == 0 {
            None
        } else if let CpEntry::ClassRef(name_index) = constant_pool.get(&super_class_index).unwrap() {
            if let CpEntry::Utf8(name) = constant_pool.get(name_index).unwrap() {
                Some(name.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    }

    // convienence methods for data from the constantpool

    pub fn get_field_ref(&self, index: &u16) -> Option<(&u16, &u16)> {
        if let CpEntry::Fieldref(class_index, name_and_type_index) = self.constant_pool.get(index).unwrap() {
            Some((class_index, name_and_type_index))
        } else {
            None
        }
    }

    pub fn get_class_ref(&self, index: &u16) -> Option<&u16> {
        if let CpEntry::ClassRef(name_index) = self.constant_pool.get(index).unwrap() {
            Some(name_index)
        } else {
            None
        }
    }

    pub fn get_utf8(&self, index: &u16) -> Option<&String> {
        if let CpEntry::Utf8(utf8) = self.constant_pool.get(index).unwrap() {
            Some(utf8)
        } else {
            None
        }
    }

    pub fn get_name_and_type(&self, index: &u16) -> Option<(&u16, &u16)> {
        if let CpEntry::NameAndType(name_index, type_index) = self.constant_pool.get(index).unwrap(){
            Some((name_index, type_index))
        } else {
            None
        }
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
    index: u16,
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
        field_index: u16,
    ) -> Self {
        Field {
            constant_pool,
            access_flags,
            name_index,
            descriptor_index,
            attributes,
            index: field_index,
        }
    }

    pub fn name(&self) -> &String {
        if let CpEntry::Utf8(utf8) = &self.constant_pool.get(&self.name_index).unwrap() {
            return utf8;
        }
        unreachable!()
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

//TODO implement more types
#[derive(Debug)]
pub enum AttributeType {
    ConstantValue(u16),
    Code(Box<MethodCode>),
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
    // variant returned for void methods
    Null,
    // 'pointer' to nothing
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BOOL(bool),
    CHAR(char),
    Ref(Arc<UnsafeCell<ObjectRef>>),
}

unsafe impl Send for Value {}

unsafe impl Sync for Value {}
