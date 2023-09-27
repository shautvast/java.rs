use std::rc::Rc;
use crate::class::Class;

pub(crate) struct Object {
    // locked: bool,
    // hashcode: i32,
    class: Rc<Class>,
    data: Vec<u8>,
}

impl Object {
    pub fn new(class: Rc<Class>, data: Vec<u8>) -> Self {
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