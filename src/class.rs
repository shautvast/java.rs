use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

use log::debug;
use rand::random;

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
            .unwrap_or(0)
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
            panic!("{:?} is not I32", self);
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
    Object(Rc<RefCell<Object>>),
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
            _ => unreachable!("not an array {:?}", self)
        }
    }
}

pub enum ArrayType {
    BOOLEAN = 4,
    CHAR = 5,
    FLOAT = 6,
    DOUBLE = 7,
    BYTE = 8,
    SHORT = 9,
    INT = 10,
    LONG = 11,
}

impl ObjectRef {
    pub fn new_object_array(class: &Class, size: usize) -> Self {
        ObjectArray(class.id, Vec::with_capacity(size))
    }

    pub fn new_array(arraytype: u8, size: usize) -> Self {
        match arraytype {
            8 => ByteArray(Vec::with_capacity(size)),
            9 => ShortArray(Vec::with_capacity(size)),
            10 => IntArray(Vec::with_capacity(size)),
            11 => LongArray(Vec::with_capacity(size)),
            6 => FloatArray(Vec::with_capacity(size)),
            7 => DoubleArray(Vec::with_capacity(size)),
            4 => BooleanArray(Vec::with_capacity(size)),
            5 => CharArray(Vec::with_capacity(size)),
            _ => unreachable!("impossible array type")
        }
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

#[derive(Debug)]
pub struct Object {
    // locked: bool,
    // hashcode: i32,
    pub id: u32,
    pub class_id: ClassId,
    pub data: Vec<Value>,
} //arrays

// object, not array
impl Object {
    pub fn new(class: &Class) -> Self {
        let instance_data = Object::init_fields(class);
        Self {
            id: random(),
            class_id: class.id,
            data: instance_data,
        }
    }

    // initializes all non-static fields to their default values
    pub(crate) fn init_fields(class: &Class) -> Vec<Value> {
        let mut field_data = vec![Value::Null;class.n_object_fields()];

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
                field_data[type_index.index] = value.into();
            }
        }

        field_data
    }

    pub fn set(&mut self, runtime_type: &Class, declared_type: &str, field_name: &str, value: Value) {
        debug!("set {:?}.{}", runtime_type.name, field_name);
        let type_index = runtime_type
            .object_field_mapping
            .get(declared_type)
            .unwrap()
            .get(field_name)
            .unwrap();
        self.data[type_index.index] = value;
    }

    pub fn get(&self, runtime_type: &Class, declared_type: &String, field_name: &String) -> &Value {
        let type_index = runtime_type
            .object_field_mapping
            .get(declared_type)
            .unwrap()
            .get(field_name)
            .unwrap();
        debug!("get {:?}:{}.{}:{} @{}", runtime_type, declared_type, field_name, type_index.type_name, type_index.index);
        debug!("from data {:?}", self.data);
        self.data.get(type_index.index).unwrap()
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
