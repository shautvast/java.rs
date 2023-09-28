use std::collections::HashMap;
use std::rc::Rc;
use crate::class::{Class, Value};

pub(crate) struct Object {
    // locked: bool,
    // hashcode: i32,
    class: Rc<Class>,
    data: HashMap<u16, Value>, //TODO optimize
}

impl Object {
    pub fn new(class: Rc<Class>, data: HashMap<u16, Value>) -> Self {
        Self {
            class,
            data,
        }
    }
}

pub(crate) struct Heap {
    objects: Vec<Object>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: vec![]
        }
    }

    pub(crate) fn new_object(&mut self, object: Object) {
        self.objects.push(object);
    }
}