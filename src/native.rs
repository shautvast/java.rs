use crate::class::{Method, UnsafeValue, Value};
use std::rc::Rc;

pub fn invoke_native(method: Rc<Method>, _args: Vec<UnsafeValue>) -> UnsafeValue {
    println!("native {}", method.name());
    Value::void()
}
