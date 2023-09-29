use std::collections::HashMap;
use std::sync::Arc;
use crate::class::{Class, Value};

#[derive(Debug)]
pub struct Object {
    // locked: bool,
    // hashcode: i32,
    _class: Arc<Class>,
    pub data: HashMap<u16, Arc<Value>>, //TODO optimize
}

unsafe impl Send for Object {}
unsafe impl Sync for Object {}

impl Object {
    pub fn new(_class: Arc<Class>, data: HashMap<u16, Arc<Value>>) -> Self {
        Self {
            _class,
            data,
        }
    }
}

pub(crate) struct Heap {
    objects: Vec<Arc<Object>>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: vec![]
        }
    }

    pub(crate) fn new_object(&mut self, object: Arc<Object>) {
        self.objects.push(object);
    }
}