use std::sync::Arc;

use crate::class::{Class, Method, UnsafeValue, Value};

pub fn invoke_native(class: Arc<Class>, method: &Method) -> UnsafeValue {
    println!("invoke native {:?}.{:?}", class.name, method.name());
    Value::void()
}
