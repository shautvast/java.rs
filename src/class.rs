use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Error;
use log::info;
use once_cell::sync::Lazy;

use crate::classloader::{CpEntry, load_class};
use crate::heap::{ObjectRef};
use crate::io::{find_class, read_bytecode, read_u16};
use crate::vm::Vm;

//trying to be ready for multithreaded as much as possible, using Arc's and all, but it will still require (a lot of) extra work
pub static mut CLASSDEFS: Lazy<HashMap<String, Arc<RefCell<Class>>>> = Lazy::new(|| HashMap::new());
//TODO add mutex..
pub static mut CLASSES: Lazy<HashMap<String, Value>> = Lazy::new(|| HashMap::new()); //TODO add mutex..

// gets the Class from cache, or reads it from classpath,
// then parses the binary data into a Class struct
// Vm keeps ownership of the class and hands out Arc references to it


pub(crate) fn get_class(
    vm: &mut Vm,
    class_name: &str,
) -> Result<Arc<RefCell<Class>>, Error> {
    info!("get_class {}", class_name);

    unsafe {
        let class = CLASSDEFS.entry(class_name.into()).or_insert_with(|| {
            // println!("read class {} ", class_name);
            let resolved_path = find_class(&vm.classpath, class_name).unwrap();
            let bytecode = read_bytecode(resolved_path).unwrap();
            let class = load_class(bytecode).unwrap();
            Arc::new(RefCell::new(class))
        });


        let clone = class.clone();
        let inited = class.borrow().inited;
        if !inited {
            // not sure why I have to create the clones first
            let clone2 = class.clone();
            let clone3 = class.clone();
            let clone4 = class.clone();
            let mut some_class = class.clone();

            if class_name != "java/lang/Class" {
                let klazz = get_class(vm, "java/lang/Class")?;
                let mut class_instance = Vm::new_instance(klazz);
                class_instance.set(&"java/lang/Class".to_owned(), &"name".to_owned(), Value::Utf8(class_name.into()));
                CLASSES.insert(class_name.into(), Value::Ref(unsafe_ref(ObjectRef::Object(Box::new(class_instance)))));
            }

            // must not enter here twice!
            clone2.borrow_mut().inited = true;

                let mut supers = vec![];
            if class_name != "java/lang/Class" {
                loop {
                    let super_class_name = some_class
                        .clone()
                        .borrow()
                        .super_class_name
                        .as_ref()
                        .map(|n| n.to_owned());
                    {
                        if let Some(super_class_name) = super_class_name {
                            if let Ok(super_class) = get_class(vm, &super_class_name) {
                                supers.push(super_class.clone());
                                some_class = super_class.clone();
                                clone4.borrow_mut().super_class = Some(super_class);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            Class::initialize_fields(clone3, supers);
            let clinit = clone2.borrow().methods.contains_key("<clinit>()V");
            let name = &clone2.borrow().name.to_owned();
            if clinit {
                vm.execute_special(name, "<clinit>()V", vec![]).unwrap();
            }
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
    pub super_classes: Vec<Type>,
    pub interface_indices: Vec<u16>,
    pub interfaces: Vec<Class>,
    pub fields: HashMap<String, Field>,
    pub methods: HashMap<String, Rc<Method>>,
    pub attributes: HashMap<String, AttributeType>,
    pub inited: bool,

    // lookup index and type from the name
    pub(crate) object_field_mapping: HashMap<String, HashMap<String, TypeIndex>>,
    pub(crate) static_field_mapping: HashMap<String, HashMap<String, TypeIndex>>,
    // static fields
    pub(crate) static_data: Vec<Value>,
}

#[derive(Debug)]
pub(crate) struct TypeIndex {
    pub type_name: String,
    pub index: usize,
}

impl TypeIndex {
    pub(crate) fn new(type_name: String, index: usize) -> Self {
        Self {
            type_name,
            index,
        }
    }
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
            super_classes: vec![],
            interface_indices,
            interfaces: vec![], // same
            fields,
            methods,
            attributes,
            inited: false,
            object_field_mapping: HashMap::new(),
            static_field_mapping: HashMap::new(),
            static_data: vec![],
        }
    }

    pub(crate) fn n_object_fields(&self) -> usize {
        self.object_field_mapping
            .iter()
            .map(|(_, v)| v.len())
            .reduce(|acc, e| acc + e)
            .unwrap()
    }

    pub(crate) fn n_static_fields(&self) -> usize {
        self.static_field_mapping
            .iter()
            .map(|(_, v)| v.len())
            .reduce(|acc, e| acc + e)
            .unwrap()
    }

    // Create a mapping per field(name) to an index in the storage vector that contains the instance data.
    // When a field is stored, first the index will be looked up, using the qualified name (from the FieldRef)
    // The qualified name is the combination of class name and field name.
    // The class name is needed as part of the key to separate class from superclass fields
    // (duplicates in the singular field name are allowed).
    // This way `this.a` can be differentiated from `super.a`.
    //
    // this method looks up this and super classes and calls map_fields for each.
    pub fn initialize_fields(class: Arc<RefCell<Class>>, super_classes: Vec<Arc<RefCell<Class>>>) {
        let mut this_field_mapping = HashMap::new();
        let mut static_field_mapping = HashMap::new();
        let mut object_field_map_index: usize = 0;
        let mut static_field_map_index: usize = 0;

        Class::add_field_mappings(
            &mut this_field_mapping,
            &mut static_field_mapping,
            class.clone(),
            &mut object_field_map_index,
            &mut static_field_map_index,
        );

        class.borrow_mut().object_field_mapping = this_field_mapping;
        class.borrow_mut().static_field_mapping = static_field_mapping;

        let static_data = Class::set_field_data(class.clone());
        class.borrow_mut().static_data = static_data;
    }

    /// for all static and non-static fields on the class compute an index
    /// the result of this function is that the class object contains mappings
    /// from the field name to the index. This index will be used to store the
    /// actual data later in a Vector.
    fn add_field_mappings(
        this_field_mapping: &mut HashMap<String, HashMap<String, TypeIndex>>,
        static_field_mapping: &mut HashMap<String, HashMap<String, TypeIndex>>,
        class: Arc<RefCell<Class>>,
        object_field_map_index: &mut usize,
        static_field_map_index: &mut usize,
    ) {
        let (o, s) = Class::map_fields(
            class.clone(),
            object_field_map_index,
            static_field_map_index,
        );
        let borrow = class.borrow();
        let name = &borrow.name;
        this_field_mapping.insert(name.to_owned(), o);
        static_field_mapping.insert(name.to_owned(), s);

        // // same for super class
        // if let Some(super_class) = class.borrow().super_class.as_ref() {
        //     Class::add_field_mappings(
        //         this_field_mapping,
        //         static_field_mapping,
        //         super_class.clone(),
        //         object_field_map_index,
        //         static_field_map_index,
        //     );
        // }
        for c in &class.borrow().super_classes {
            Class::add_field_mappings(
                this_field_mapping,
                static_field_mapping,
                c.clone(),
                object_field_map_index,
                static_field_map_index,
            );
        }
    }

    // part of the initialize procedure
    /// here the actual indices are created
    fn map_fields(
        class: Arc<RefCell<Class>>,
        object_field_map_index: &mut usize,
        static_field_map_index: &mut usize,
    ) -> (
        HashMap<String, TypeIndex>,
        HashMap<String, TypeIndex>,
    ) {
        let mut this_fields = HashMap::new(); //fields in class are stored per class and every superclass.
        let mut static_fields = HashMap::new(); //fields in class are stored per class and every superclass.

        for (name, field) in &class.borrow().fields {
            if field.is(Modifier::Static) {
                static_fields.insert(
                    name.to_owned(),
                    TypeIndex::new(field.type_of().to_owned(), *static_field_map_index),
                );
                *static_field_map_index += 1;
            } else {
                this_fields.insert(
                    name.to_owned(),
                    TypeIndex::new(field.type_of().to_owned(), *object_field_map_index),
                ); //name => (type,index)
                *object_field_map_index += 1;
            }
        }
        (this_fields, static_fields)
    }

    /// the bytecode version
    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    /// get a method by signature
    pub fn get_method(&self, name: &str) -> Option<&Rc<Method>> {
        self.methods.get(name)
    }

    /// get the class name
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

    /// creates default values for every field, ie null for objects, 0 for integers etc
    /// this is the step before the constructor/static initializer can be called to set hardcoded
    /// or computed values.
    pub(crate) fn set_field_data(class: Arc<RefCell<Class>>) -> Vec<Value> {
        let mut field_data = vec![Value::Null; class.borrow().n_static_fields()];

        for (_, this_class) in &class.borrow().static_field_mapping {
            for (_name, type_index) in this_class {
                let value = match type_index.type_name.as_str() {
                    "Z" => Value::BOOL(false),
                    "B" => Value::I32(0),
                    "S" => Value::I32(0),
                    "I" => Value::I32(0),
                    "J" => Value::I64(0),
                    "F" => Value::F32(0.0),
                    "D" => Value::F64(0.0),
                    _ => Value::Null,
                };
                // println!("{} = {:?}", name, value);
                field_data[type_index.index] = value.into();
            }
        }

        field_data
    }

    // convienence methods for data from the constantpool

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

const _MODIFIERS: [(Modifier, &str); 12] = [
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

#[derive(Debug, Clone)]
pub enum Value {
    // variant returned for void methods
    Void,
    // 'pointer' to nothing
    Null,
    // primitives
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BOOL(bool),
    CHAR(char),
    // objects and arrays
    Ref(UnsafeRef),
    // special object
    Utf8(String),
}

impl Value {
    // panics if not correct type
    pub fn into_i32(self) -> i32 {
        if let Value::I32(v) = self {
            v
        } else {
            panic!();
        }
    }

    pub fn into_object(self) -> UnsafeRef {
        if let Value::Ref(v) = self {
            v
        } else {
            panic!();
        }
    }
}

pub type UnsafeRef = Arc<UnsafeCell<ObjectRef>>;

pub fn unsafe_ref(object: ObjectRef) -> UnsafeRef {
    return Arc::new(UnsafeCell::new(object));
}

// pub fn unsafe_val(val: Value) -> UnsafeValue {
//     return Arc::new(UnsafeCell::new(val));
// }

pub fn type_ref(class: Class) -> Type {
    return Arc::new(RefCell::new(class));
}

pub type Type = Arc<RefCell<Class>>;

unsafe impl Send for Value {}

unsafe impl Sync for Value {}
