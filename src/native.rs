use std::rc::Rc;
use crate::class::{Method, UnsafeValue, Value};

pub fn invoke_native(method: Rc<Method>, args: Vec<UnsafeValue>) -> UnsafeValue {
    println!("native {}", method.name());
    Value::void()
}


