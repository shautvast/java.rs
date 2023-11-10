use anyhow::Error;
use log::debug;

use crate::class::Value;

#[derive(Debug)]
pub struct StackFrame {
    pub(crate) at: String,
    pub(crate) data: Vec<Value>,
}

// maybe just call frame
impl StackFrame {
    pub(crate) fn new(at_class: &str, at_method: &str) -> Self {
        let mut at: String = at_class.into();
        at.push('.');
        at.push_str(at_method);
        Self { at, data: vec![] }
    }

    pub(crate) fn push(&mut self, val: Value) {
        debug!("push {:?}", val);
        self.data.push(val);
    }

    pub(crate) fn pop(&mut self) -> Result<Value, Error> {
        Ok(self.data.pop().unwrap())
    }
}