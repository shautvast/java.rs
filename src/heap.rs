use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use crate::class::{Class, Value};

#[derive(Debug)]
pub struct Object {
    // locked: bool,
    // hashcode: i32,
    class: Arc<Class>,
    pub data: HashMap<u16, Arc<Value>>, //TODO optimize
}

impl Object {
    pub fn new(class: Arc<Class>, data: HashMap<u16, Arc<Value>>) -> Self {
        Self {
            class,
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