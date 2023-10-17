use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Error};
use once_cell::sync::Lazy;

use crate::classloader::{CpEntry, load_class};
use crate::heap::ObjectRef;
use crate::io::{find_class, read_bytecode, read_u16};
use crate::vm::Vm;

//trying to be ready for multithreaded as much as possible, using Arc's and all, but it will still require (a lot of) extra work
static mut CLASSDEFS: Lazy<HashMap<String, Arc<RefCell<Class>>>> = Lazy::new(|| HashMap::new()); //TODO add mutex..


// gets the Class from cache, or reads it from classpath,
// then parses the binary data into a Class struct
// Vm keeps ownership of the class and hands out Arc references to it
pub fn get_class(vm: &mut Vm, calling_class_name: Option<&str>, class_name: &str) -> Result<Arc<RefCell<Class>>, Error> {
    println!("get_class {}", class_name);

    unsafe {
        // not pretty...sorry
        if let Some(calling_class_name) = calling_class_name {
            if class_name == calling_class_name { // works around the situation that static initializer needs a ref to the class it's in
                panic!();
                // return Ok(CLASSDEFS.get(class_name.into()).unwrap().clone()); // in that case the class is guaranteed to be here
            }
        }

        let new_class = CLASSDEFS.entry(class_name.into()).or_insert_with(|| {
            println!("read class {} ", class_name);
            let resolved_path = find_class(&vm.classpath, class_name).unwrap();
            // println!("full path {}", resolved_path);
            let bytecode = read_bytecode(resolved_path).unwrap();
            let mut class = load_class(bytecode).unwrap();
            let super_class_name = class.super_class_name.as_ref();
            if let Some(super_class_name) = super_class_name {
                if let Ok(super_class) = get_class(vm, Some(class_name), &super_class_name) {
                    class.super_class = Some(super_class);
                } else {
                    unreachable!()
                }
            }

            let class = Arc::new(RefCell::new(class));
            Class::initialize_fields(class.clone());
            class
        });


        // calling clinit before the end of this function has been a PITA
        // 1. infinite recursion
        // panic after second borrow.
        // the problem is pretty fundamental: method (clinit) should be called before the class is returned,
        // but the executing code needs a reference to itself. So get_class is called recursively, but clinit must be called exactly once!
        // putting the call to clinit in the closure above is way nicer, but the signature change (wrap it in Arc<RefCell>)
        //update: this is probably not needed anymore because of the check in PUTSTATIC

        //somehow this clone needs to be there before clinit is called, even though the newclass ref remains in scope
        let clone = new_class.clone();

        if  new_class.clone().borrow().methods.contains_key("<clinit>()V") {
            vm.execute_class(new_class.clone(), "<clinit>()V", vec![]).unwrap();
        }

        Ok(clone)
    }
}


/// the class definition as read from the class file + derived values
// TODO implement call to static initializers
#[derive(Debug)]
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Rc<HashMap<u16, CpEntry>>,
    pub access_flags: u16,
    pub name: String,
    pub super_class_name: Option<String>,
    pub super_class: Option<Type>,
    pub interface_indices: Vec<u16>,
    pub interfaces: Vec<Class>,
    pub fields: HashMap<String, Field>,
    pub methods: HashMap<String, Rc<Method>>,
    pub attributes: HashMap<String, AttributeType>,
    pub(crate) object_field_mapping: HashMap<String, HashMap<String, (String, usize)>>,
    // first key: this/super/supersuper-name(etc), second key: fieldname, value (type, index)
    pub(crate) static_field_mapping: HashMap<String, HashMap<String, (String, usize)>>,
    // first key: this/super/supersuper-name(etc), second key: fieldname, value (type, index)
    pub(crate) static_data: Vec<Option<UnsafeValue>>,
}

impl Class {
    pub fn new(
        minor_version: u16,
        major_version: u16,
        constant_pool: Rc<HashMap<u16, CpEntry>>,
        access_flags: u16,
        this_class: u16,
        super_class_index: u16,
        interface_indices: Vec<u16>,
        fields: HashMap<String, Field>,
        methods: HashMap<String, Rc<Method>>,
        attributes: HashMap<String, AttributeType>,
    ) -> Self {
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
            object_field_mapping: HashMap::new(),
            static_field_mapping: HashMap::new(),
            static_data: vec![],
        }
    }

    pub(crate) fn n_object_fields(&self) -> usize {
        self.object_field_mapping.iter().map(|(_, v)| v.len()).reduce(|acc, e| acc + e).unwrap()
    }

    pub(crate) fn n_static_fields(&self) -> usize {
        self.static_field_mapping.iter().map(|(_, v)| v.len()).reduce(|acc, e| acc + e).unwrap()
    }

    // Create a mapping per field(name) to an index in the storage vector that contains the instance data.
    // When a field is stored, first the index will be looked up, using the qualified name (from the FieldRef)
    // The qualified name is the combination of class name and field name.
    // The class name is needed as part of the key to separate class from superclass fields
    // (duplicates in the singular field name are allowed).
    // This way `this.a` can be differentiated from `super.a`.
    //
    // this method looks up this and super classes and calls map_fields for each.
    pub fn initialize_fields(class: Arc<RefCell<Class>>) {
        let mut this_field_mapping = HashMap::new();
        let mut static_field_mapping = HashMap::new();
        let mut object_field_map_index: usize = 0;
        let mut static_field_map_index: usize = 0;

        Class::add_field_mappings(&mut this_field_mapping, &mut static_field_mapping, class.clone(), &mut object_field_map_index, &mut static_field_map_index);

        class.borrow_mut().object_field_mapping = this_field_mapping;
        class.borrow_mut().static_field_mapping = static_field_mapping;

        let static_data = Class::set_field_data(class.clone());
        class.borrow_mut().static_data = static_data;
    }

    fn add_field_mappings(this_field_mapping: &mut HashMap<String, HashMap<String, (String, usize)>>,
                          static_field_mapping: &mut HashMap<String, HashMap<String, (String, usize)>>,
                          class: Arc<RefCell<Class>>,
                          object_field_map_index: &mut usize,
                          static_field_map_index: &mut usize) {
        let (o, s) = Class::map_fields(class.clone(), object_field_map_index, static_field_map_index);
        let borrow = class.borrow();
        let name = &borrow.name;
        this_field_mapping.insert(name.to_owned(), o);
        static_field_mapping.insert(name.to_owned(), s);

        if let Some(super_class) = class.borrow().super_class.as_ref() {
            Class::add_field_mappings(this_field_mapping, static_field_mapping, super_class.clone(), object_field_map_index, static_field_map_index);
        }
    }

    // part of the initialize procedure
    fn map_fields(
        class: Arc<RefCell<Class>>,
        object_field_map_index: &mut usize,
        static_field_map_index: &mut usize,
    ) -> (HashMap<String, (String, usize)>, HashMap<String, (String, usize)>) {
        let mut this_fields = HashMap::new(); //fields in class are stored per class and every superclass.
        let mut static_fields = HashMap::new(); //fields in class are stored per class and every superclass.

        for (name, field) in &class.borrow().fields {
            if field.is(Modifier::Static) {
                static_fields.insert(
                    name.to_owned(),
                    (field.type_of().to_owned(), *static_field_map_index),
                );
                *static_field_map_index += 1;
            } else {
                this_fields.insert(
                    name.to_owned(),
                    (field.type_of().to_owned(), *object_field_map_index),
                ); //name => (type,index)
                *object_field_map_index += 1;
            }
        }
        (this_fields, static_fields)
    }

    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    pub fn get_method(&self, name: &str) -> Result<&Rc<Method>, Error> {
        self.methods
            .get(name)
            .ok_or(anyhow!("Method {} not found", name))
    }

    fn class_name(
        super_class_index: u16,
        constant_pool: Rc<HashMap<u16, CpEntry>>,
    ) -> Option<String> {
        if super_class_index == 0 {
            None
        } else if let CpEntry::ClassRef(name_index) = constant_pool.get(&super_class_index).unwrap()
        {
            if let CpEntry::Utf8(name) = constant_pool.get(name_index).unwrap() {
                Some(name.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn set_field_data(class: Arc<RefCell<Class>>) -> Vec<Option<UnsafeValue>> {
        let mut field_data = vec![None; class.borrow().n_static_fields()];

        for (_, this_class) in &class.borrow().static_field_mapping {
            for (name, (fieldtype, index)) in this_class {
                let value = match fieldtype.as_str() {
                    "Z" => Value::BOOL(false),
                    "B" => Value::I32(0),
                    "S" => Value::I32(0),
                    "I" => Value::I32(0),
                    "J" => Value::I64(0),
                    "F" => Value::F32(0.0),
                    "D" => Value::F64(0.0),
                    _ => Value::Null,
                };
                println!("{} = {:?}", name, value);
                field_data[*index] = Some(value.into());
            }
        }

        field_data
    }

    // convienence methods for data from the constantpool

    pub fn cp_field_ref(&self, index: &u16) -> Option<(&u16, &u16)> {
        if let CpEntry::Fieldref(class_index, name_and_type_index) =
            self.constant_pool.get(index).unwrap()
        {
            Some((class_index, name_and_type_index))
        } else {
            None
        }
    }

    /// both methodRef and InterfaceMethodRef
    /// returns (class_index, name_and_type_index)
    pub fn cp_method_ref(&self, index: &u16) -> Option<(&u16, &u16)> {
        if let CpEntry::MethodRef(class_index, name_and_type_index)
        | CpEntry::InterfaceMethodref(class_index, name_and_type_index) =
            self.constant_pool.get(index).unwrap()
        {
            Some((class_index, name_and_type_index))
        } else {
            None
        }
    }

    pub fn cp_class_ref(&self, index: &u16) -> Option<&u16> {
        if let CpEntry::ClassRef(name_index) = self.constant_pool.get(index).unwrap() {
            Some(name_index)
        } else {
            None
        }
    }

    pub fn cp_utf8(&self, index: &u16) -> Option<&String> {
        if let CpEntry::Utf8(utf8) = self.constant_pool.get(index).unwrap() {
            Some(utf8)
        } else {
            None
        }
    }

    pub fn cp_name_and_type(&self, index: &u16) -> Option<(&u16, &u16)> {
        if let CpEntry::NameAndType(name_index, type_index) = self.constant_pool.get(index).unwrap()
        {
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

    pub fn is(&self, modifier: Modifier) -> bool {
        let m = modifier as u16;
        (self.access_flags & m) == m
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

const MODIFIERS: [(Modifier, &str); 12] = [
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
    Ref(UnsafeRef),
    Utf8(String),
}

impl Value {
    pub fn void() -> UnsafeValue {
        Arc::new(UnsafeCell::new(Value::Void))
    }
}

impl Into<UnsafeValue> for Value {
    fn into(self) -> UnsafeValue {
        Arc::new(UnsafeCell::new(self))
    }
}

pub type UnsafeValue = Arc<UnsafeCell<Value>>;

pub type UnsafeRef = Arc<UnsafeCell<ObjectRef>>;

pub fn unsafe_ref(object: ObjectRef) -> UnsafeRef{
    return Arc::new(UnsafeCell::new(object))
}

pub fn unsafe_val(val: Value) -> UnsafeValue{
    return Arc::new(UnsafeCell::new(val))
}

pub fn type_ref(class: Class) -> Type{
    return Arc::new(RefCell::new(class))
}
pub type Type = Arc<RefCell<Class>>;

unsafe impl Send for Value {}

unsafe impl Sync for Value {}
