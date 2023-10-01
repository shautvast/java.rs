use crate::class::{Class, Value};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use crate::classloader::CpEntry;

pub struct Object {
    // locked: bool,
    // hashcode: i32,
    pub class: Rc<Class>,
    pub data: HashMap<u16, Rc<Value>>, //TODO optimize
}

unsafe impl Send for Object {}

unsafe impl Sync for Object {}

impl Object {
    pub fn new(class: Rc<Class>, data: HashMap<u16, Rc<Value>>) -> Self {
        Self { class, data }
    }

    fn get_field(&self, cp_index: &u16) -> &str {
        if let CpEntry::Utf8(name) = self.class.constant_pool.get(cp_index).unwrap() {
            return name;
        }
        panic!()
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields: Vec<String> = self.data.iter().map(|(k, v)| {
            let mut r: String = self.get_field(k).into();
            r.push(':');
            r.push_str(format!("{:?}", v).as_str());
            r
        }
        ).collect();
        write!(
            f,
            "{} {{ {:?} }}",
            self.class.get_name(), fields
        )
    }
}

pub(crate) struct Heap {
    objects: Vec<Rc<Object>>,
}

impl Heap {
    pub fn new() -> Self {
        Self { objects: vec![] }
    }

    pub(crate) fn new_object(&mut self, object: Rc<Object>) {
        self.objects.push(object);
    }
}
