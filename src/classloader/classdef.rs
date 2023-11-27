use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::classloader::io::read_u16;
use crate::vm::opcodes::Opcode;

/// This is the class representation when the bytecode had just been loaded.

pub(crate) struct ClassDef {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Rc<HashMap<u16, CpEntry>>,
    pub access_flags: u16,
    this_class: u16,
    pub super_class: Option<u16>,
    pub interfaces: Vec<u16>,
    pub fields: HashMap<String, Field>,
    pub methods: HashMap<String, Method>,
    pub attributes: HashMap<String, AttributeType>,
}

impl Debug for ClassDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cp_class_name(&self.this_class))
    }
}

impl ClassDef {
    pub fn new(
        minor_version: u16,
        major_version: u16,
        constant_pool: Rc<HashMap<u16, CpEntry>>,
        access_flags: u16,
        this_class: u16,
        super_class: Option<u16>,
        interfaces: Vec<u16>,
        fields: HashMap<String, Field>,
        methods: HashMap<String, Method>,
        attributes: HashMap<String, AttributeType>,
    ) -> Self {
        Self {
            major_version,
            minor_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        }
    }
    /// the bytecode version
    pub fn version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    /// get the class name
    pub fn name(&self) -> &str {
        self.cp_class_name(&self.this_class)
    }

    pub fn get_method(&self, name: &str) -> Option<&Method> {
        self.methods.get(name)
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.methods.contains_key(name)
    }

    pub fn cp_field_ref(&self, index: &u16) -> (&u16, &u16) {
        if let CpEntry::Fieldref(class_index, name_and_type_index) =
            self.constant_pool.get(index).unwrap()
        {
            (class_index, name_and_type_index)
        } else {
            unreachable!("should be field")
        }
    }

    /// both methodRef and InterfaceMethodRef
    /// returns (class_index, name_and_type_index)
    pub fn cp_method_ref(&self, index: &u16) -> (&u16, &u16) {
        if let CpEntry::MethodRef(class_index, name_and_type_index)
        | CpEntry::InterfaceMethodref(class_index, name_and_type_index) =
            self.constant_pool.get(index).unwrap()
        {
            (class_index, name_and_type_index)
        } else {
            unreachable!("should be method")
        }
    }

    pub fn cp_class_name(&self, index: &u16) -> &String {
        let cr = self.cp_class_ref(index);
        self.cp_utf8(cr)
    }

    pub fn cp_class_ref(&self, index: &u16) -> &u16 {
        if let CpEntry::ClassRef(name_index) = self.constant_pool.get(index).unwrap() {
            name_index
        } else {
            unreachable!("should be class entry")
        }
    }

    pub fn cp_utf8(&self, index: &u16) -> &String {
        if let CpEntry::Utf8(utf8) = self.constant_pool.get(index).unwrap() {
            utf8
        } else {
            unreachable!("should be utf8 entry")
        }
    }

    pub fn cp_name_and_type(&self, index: &u16) -> (&u16, &u16) {
        if let CpEntry::NameAndType(name_index, type_index) = self.constant_pool.get(index).unwrap()
        {
            (name_index, type_index)
        } else {
            unreachable!("should be name_and_type entry")
        }
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
    // (utf8)
    StringRef(u16),
    // (utf8)
    Fieldref(u16, u16),
    // (class, name_and_type)
    MethodRef(u16, u16),
    // (class, name_and_type)
    InterfaceMethodref(u16, u16),
    // (class, name_and_type)
    NameAndType(u16, u16),
    // (name, descriptor)
    MethodHandle(u8, u16),
    MethodType(u16),
    InvokeDynamic(u16, u16),
}

pub enum Modifier {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Synchronized = 0x0020,
    Volatile = 0x0040,
    Transient = 0x0080,
    Native = 0x0100,
    Abstract = 0x0400,
    Strict = 0x0800,
    Synthetic = 0x1000,
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

pub(crate) const _MODIFIERS: [(Modifier, &str); 12] = [
    (Modifier::Public, "public "),
    (Modifier::Private, "private "),
    (Modifier::Protected, "protected "),
    (Modifier::Static, "static "),
    (Modifier::Final, "final "),
    (Modifier::Synchronized, "synchronized "),
    (Modifier::Volatile, "volatile "),
    (Modifier::Transient, "transient "),
    (Modifier::Native, "native "),
    (Modifier::Abstract, "abstract"),
    (Modifier::Strict, "strict"),
    (Modifier::Synthetic, "synthetic"),
];

pub struct Field {
    constant_pool: Rc<HashMap<u16, CpEntry>>,
    access_flags: u16,
    pub(crate) name_index: u16,
    descriptor_index: u16,
    attributes: HashMap<String, AttributeType>,
    _index: u16,
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
            _index: field_index,
        }
    }

    pub fn is(&self, modifier: Modifier) -> bool {
        let modifier = modifier as u16;
        self.access_flags & modifier == modifier
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

pub struct Method {
    pub(crate) constant_pool: Rc<HashMap<u16, CpEntry>>,
    pub access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    pub(crate) attributes: HashMap<String, AttributeType>,
    pub(crate) code: Vec<Opcode>,
}

impl Debug for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Method {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
            self.access_flags, self.name_index, self.descriptor_index, self.attributes
        )
    }
}

impl Method {
    pub(crate) fn new(
        constant_pool: Rc<HashMap<u16, CpEntry>>,
        access_flags: u16,
        name_index: u16,
        descriptor_index: u16,
        attributes: HashMap<String, AttributeType>,
        code: Vec<Opcode>,
    ) -> Self {
        Method {
            constant_pool,
            access_flags,
            name_index,
            descriptor_index,
            attributes,
            code,
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

    pub fn is(&self, modifier: Modifier) -> bool {
        let m = modifier as u16;
        (self.access_flags & m) == m
    }
}
