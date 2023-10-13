use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::class::{Class, Value};
use crate::classloader::CpEntry;

// trying to implement efficient object instance storage
pub struct Object {
    // locked: bool,
    // hashcode: i32,
    pub class: Rc<Class>,
    pub data: Option<Vec<Arc<UnsafeCell<Value>>>>,
}//arrays

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
    ObjectArray(Vec<Arc<UnsafeCell<ObjectRef>>>),
    Object(Box<Object>),
}

unsafe impl Send for Object {}

unsafe impl Sync for Object {}

// object, not array
impl Object {
    pub fn new(class: Rc<Class>) -> Self {
        Self { class, data: None }
    }

    // initializes all non-static fields to their default values
    pub(crate) fn init_fields(&mut self) {
        let mut field_data = Vec::with_capacity(self.class.n_fields());

        for (_, fields) in self.class.field_mapping.as_ref().unwrap() {
            for (_, (fieldtype, _)) in fields {
                let value = match fieldtype.as_str() {
                    "Z" => Value::BOOL(false),
                    "B" => Value::I32(0),
                    "S" => Value::I32(0),
                    "I" => Value::I32(0),
                    "J" => Value::I64(0),
                    "F" => Value::F32(0.0),
                    "D" => Value::F64(0.0),
                    "L" => Value::Null,
                    _ => Value::Void,
                };
                field_data.push(Arc::new(UnsafeCell::new(value)));
            }
        }

        self.data = Some(field_data);
    }

    pub fn set(&mut self, class_name: &String, field_name: &String, value: Arc<UnsafeCell<Value>>) {
        let p = self.class.field_mapping.as_ref().unwrap();
        let p2 = p.get(class_name).unwrap();
        let (_type, index) = p2.get(field_name).unwrap();
        self.data.as_mut().unwrap()[*index] = value;
    }

    pub fn get(&mut self, class_name: &String, field_name: &String) -> &Arc<UnsafeCell<Value>> {
        let (_type, index) = &self.class.field_mapping.as_ref().unwrap().get(class_name).unwrap().get(field_name).unwrap(); // (index, type)
        &self.data.as_ref().unwrap()[*index]
    }

    fn get_field_name(&self, cp_index: &u16) -> &str {
        if let CpEntry::Utf8(name) = self.class.constant_pool.get(cp_index).unwrap() {
            return name;
        }
        panic!()
    }
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
        write!(
            f,
            "{}",
            self.class.name
        )
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
