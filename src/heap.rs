use std::cell::{RefCell, UnsafeCell};
use std::fmt;
use std::sync::Arc;

use crate::class::{Class, Type, UnsafeValue, Value};
use crate::heap::ObjectRef::{IntArray, ObjectArray};

// can contain object or array
#[derive(Debug)]
pub enum ObjectRef {
    ByteArray(Vec<i8>),
    ShortArray(Vec<i16>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
    BooleanArray(Vec<bool>),
    CharArray(Vec<char>),
    ObjectArray(Type, Vec<Arc<UnsafeCell<ObjectRef>>>),
    Object(Box<Object>),
}

impl ObjectRef {
    pub fn new_object_array(class: Type, size: usize) -> Self {
        ObjectArray(class, Vec::with_capacity(size))
    }

    pub fn new_int_array(size: usize) -> Self {
        IntArray(Vec::with_capacity(size))
    }
}

// trying to implement efficient object instance storage
pub struct Object {
    // locked: bool,
    // hashcode: i32,
    pub class: Arc<RefCell<Class>>,
    pub data: Vec<UnsafeValue>,
} //arrays

unsafe impl Send for Object {}

unsafe impl Sync for Object {}

// object, not array
impl Object {
    pub fn new(class: Arc<RefCell<Class>>) -> Self {
        let instance_data = Object::init_fields(class.clone());
        Self {
            class,
            data: instance_data,
        }
    }

    // initializes all non-static fields to their default values
    pub(crate) fn init_fields(class: Arc<RefCell<Class>>) -> Vec<UnsafeValue> {
        let mut field_data = Vec::with_capacity(class.borrow().n_object_fields());

        for (_, fields) in &class.borrow().object_field_mapping {
            for (_, (fieldtype, _)) in fields {
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
                field_data.push(value.into());
            }
        }

        field_data
    }

    pub fn set(&mut self, class_name: &String, field_name: &String, value: UnsafeValue) {
        let borrow = self.class.borrow();
        let (_type, index) = borrow
            .object_field_mapping
            .get(class_name)
            .unwrap()
            .get(field_name)
            .unwrap();
        self.data[*index] = value;
    }

    pub fn get(&mut self, class_name: &String, field_name: &String) -> &UnsafeValue {
        let borrow = self.class.borrow();
        let (_type, index) = borrow
            .object_field_mapping
            .get(class_name)
            .unwrap()
            .get(field_name)
            .unwrap();
        &self.data[*index]
    }

    // fn get_field_name(&self, cp_index: &u16) -> &str {
    //     if let CpEntry::Utf8(name) = self.class.constant_pool.get(cp_index).unwrap() {
    //         return name;
    //     }
    //     panic!()
    // }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let fields: Vec<String> = self.data.unwrap().iter().map(|(k)| {
        //     // let mut r: String = self.get_field_name(k).into();
        //     // r.push(':');
        //     // r.push_str(format!("{:?}").as_str());
        //     // r
        // }
        // ).collect();
        write!(f, "{}", self.class.borrow().name)
    }
}

// will using Arc's enable a GC-less heap????
pub(crate) struct Heap {
    objects: Vec<Arc<UnsafeCell<ObjectRef>>>,
}

impl Heap {
    pub fn new() -> Self {
        Self { objects: vec![] }
    }

    pub(crate) fn new_object(&mut self, object: Arc<UnsafeCell<ObjectRef>>) {
        self.objects.push(object);
    }
}
