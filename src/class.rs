use std::collections::{HashMap, LinkedList};

use crate::class::ObjectRef::*;

pub type ClassId = usize;

#[derive(Debug, Clone)]
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

// could move classdef into here. Saves one hashmap lookup
// have to look at ownership though
#[derive(Debug, Clone)]
pub struct Class {
    pub id: ClassId,
    pub initialized: bool,
    pub name: String,
    pub superclass: Option<ClassId>,
    pub parents: LinkedList<ClassId>,
    pub interfaces: Vec<ClassId>,
    // lookup index and type from the name
    pub(crate) object_field_mapping: HashMap<String, HashMap<String, TypeIndex>>,
    pub(crate) static_field_mapping: HashMap<String, HashMap<String, TypeIndex>>,
    // pub(crate) static_field_data: Vec<Value>,
}

impl Class {
    pub(crate) fn n_object_fields(&self) -> usize {
        self.object_field_mapping
            .iter()
            .map(|(_, v)| v.len())
            .reduce(|acc, e| acc + e)
            .unwrap()
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
    Ref(ObjectRef),
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

    pub fn into_object(self) -> ObjectRef {
        if let Value::Ref(v) = self {
            v
        } else {
            panic!();
        }
    }
}

#[derive(Debug, Clone)]
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
    ObjectArray(ClassId, Vec<ObjectRef>),
    Object(Object),
    //Box necessary??
    Class(Class),
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

impl ObjectRef {
    pub fn new_object_array(class: &Class, size: usize) -> Self {
        ObjectArray(class.id, Vec::with_capacity(size))
    }

    pub fn new_int_array(size: usize) -> Self {
        IntArray(Vec::with_capacity(size))
    }

    pub fn new_byte_array(d: Vec<u8>) -> Self {
        ByteArray(into_vec_i8(d))
    }
}

fn into_vec_i8(v: Vec<u8>) -> Vec<i8> {
    let mut v = std::mem::ManuallyDrop::new(v);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

#[derive(Debug, Clone)]
pub struct Object {
    // locked: bool,
    // hashcode: i32,
    pub class_id: ClassId,
    pub data: Vec<Value>,
} //arrays

// object, not array
impl Object {
    pub fn new(class: &Class) -> Self {
        let instance_data = Object::init_fields(class);
        Self {
            class_id: class.id,
            data: instance_data,
        }
    }

    // initializes all non-static fields to their default values
    pub(crate) fn init_fields(class: &Class) -> Vec<Value> {
        let mut field_data = Vec::with_capacity(class.n_object_fields());

        for (_, fields) in &class.object_field_mapping {
            for (_, type_index) in fields {
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
                field_data.push(value.into());
            }
        }

        field_data
    }

    pub fn set(&mut self, class: &Class, declared_type: &str, field_name: &str, value: Value) {
        let type_index = class
            .object_field_mapping
            .get(declared_type)
            .unwrap()
            .get(field_name)
            .unwrap();
        self.data[type_index.index] = value;
    }

    pub fn get(&mut self, instancedef: &Class, declared_type: &String, field_name: &String) -> &Value {
        let type_index = instancedef
            .object_field_mapping
            .get(declared_type)
            .unwrap()
            .get(field_name)
            .unwrap();
        &self.data[type_index.index]
    }

    // fn get_field_name(&self, cp_index: &u16) -> &str {
    //     if let CpEntry::Utf8(name) = self.class.constant_pool.get(cp_index).unwrap() {
    //         return name;
    //     }
    //     panic!()
    // }
}

// impl Debug for Object {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // let fields: Vec<String> = self.data.unwrap().iter().map(|(k)| {
//         //     // let mut r: String = self.get_field_name(k).into();
//         //     // r.push(':');
//         //     // r.push_str(format!("{:?}").as_str());
//         //     // r
//         // }
//         // ).collect();
//         write!(f, "{}", self.class.name)
//     }
// }
