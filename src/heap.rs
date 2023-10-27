use std::cell::{RefCell, UnsafeCell};
use std::fmt;
use std::fmt::{Debug, Formatter, write};
use std::sync::Arc;
use ObjectRef::{BooleanArray, CharArray, DoubleArray, FloatArray, LongArray, ShortArray};

use crate::class::{Class, Type, UnsafeValue, Value};
use crate::heap::ObjectRef::{ByteArray, IntArray, ObjectArray, StringArray};

// can contain object or array
pub enum ObjectRef {
    ByteArray(Vec<i8>),
    ShortArray(Vec<i16>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
    BooleanArray(Vec<bool>),
    CharArray(Vec<char>),
    StringArray(Vec<String>),
    ObjectArray(Type, Vec<Arc<UnsafeCell<ObjectRef>>>),
    Object(Box<Object>),
    Class(Arc<RefCell<Class>>),
}

impl ObjectRef {
    pub fn get_array_length(&self) -> usize {
        match self {
            ByteArray(d) => d.len(),
            ShortArray(d) => d.len(),
            IntArray(d) => d.len(),
            LongArray(d) => d.len(),
            FloatArray(d) => d.len(),
            DoubleArray(d) => d.len(),
            BooleanArray(d) => d.len(),
            CharArray(d) => d.len(),
            StringArray(d) => d.len(),
            ObjectArray(_, d) => d.len(),
            _ => unreachable!("not an array")
        }
    }
}

fn into_vec_i8(v: Vec<u8>) -> Vec<i8> {
    // ideally we'd use Vec::into_raw_parts, but it's unstable,
    // so we have to do it manually:

    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = std::mem::ManuallyDrop::new(v);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

impl ObjectRef {
    pub fn new_object_array(class: Type, size: usize) -> Self {
        ObjectArray(class, Vec::with_capacity(size))
    }

    pub fn new_int_array(size: usize) -> Self {
        IntArray(Vec::with_capacity(size))
    }

    pub fn new_byte_array(d: Vec<u8>) -> Arc<UnsafeCell<Self>> {
        Arc::new(UnsafeCell::new(ByteArray(into_vec_i8(d))))
    }
}

impl Debug for ObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BooleanArray(d) => write!(f, "[Z;{}]", d.len()),
            ByteArray(d) => write!(f, "[B;{}]", d.len()),
            CharArray(d) => write!(f, "[C;{}]", d.len()),
            DoubleArray(d) => write!(f, "[D;{}]", d.len()),
            FloatArray(d) => write!(f, "[F;{}]", d.len()),
            IntArray(d) => write!(f, "[I;{}]", d.len()),
            LongArray(d) => write!(f, "[J;{}]", d.len()),
            ObjectArray(t, d) => write!(f, "[L{};{}]", t.borrow().name, d.len()),
            ShortArray(d) => write!(f, "[S;{}]", d.len()),
            StringArray(d) => write!(f, "[S;{}]", d.len()),
            ObjectRef::Object(r) => write!(f, "{}{{ {:?} }}", r.class.borrow().name, r.data),
            ObjectRef::Class(s) => write!(f, "Class {:?}", s.borrow().name),
        }
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
