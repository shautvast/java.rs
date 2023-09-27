use std::collections::HashMap;
use std::rc::Rc;
use crate::{CpEntry, opcodes};
use crate::heap::{Heap, Object};
use crate::io::*;
use crate::class::{AttributeType, Class, Method, Value};

struct StackFrame {
    data: Vec<Value>,
}

impl StackFrame {
    fn new() -> Self {
        Self {
            data: vec![]
        }
    }

    fn push(&mut self, val: Value) {
        self.data.push(val);
    }

    fn pop(&mut self) -> Option<Value> {
        self.data.pop()
    }
}

pub struct Vm {
    classes: HashMap<String, Class>,
    heap: Heap,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            heap: Heap::new(),
        }
    }

    pub fn new_instance(&self, class: &Class) {
        for f in &class.fields {
            println!("{}", f.type_of());
        }
        // Object::new(Rc::new(class))
    }

    pub fn execute(&mut self, method: &Method) -> Option<Value> {
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let mut stack = StackFrame::new();
            let mut pc: usize = 0;
            while pc < code.opcodes.len() {
                let opcode = &code.opcodes[pc];
                pc += 1;
                println!("{}", opcode);
                match opcode {
                    &opcodes::BIPUSH => {
                        let c = code.opcodes[pc] as i32;
                        stack.push(Value::I32(c));
                        pc += 1;
                    }
                    &opcodes::LDC => {
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                stack.push(Value::I32(*i));
                            }
                            CpEntry::Float(f) => {
                                stack.push(Value::F32(*f));
                            }
                            _ => {}
                        }
                        pc += 1;
                    }
                    &opcodes::LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                stack.push(Value::I32(*i));
                            }
                            CpEntry::Float(f) => {
                                stack.push(Value::F32(*f));
                            }
                            _ => { panic!("unexpected") }
                        }
                        pc += 2;
                    }
                    &opcodes::LDC2_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Double(d) => {
                                stack.push(Value::F64(*d));
                            }
                            CpEntry::Long(l) => {
                                stack.push(Value::I64(*l));
                            }
                            _ => { panic!("unexpected") }
                        }

                        pc += 2;
                    }
                    &opcodes::ALOAD_0 => {}
                    &opcodes::IRETURN => {
                        return stack.pop();
                    }
                    &opcodes::DRETURN => {
                        return stack.pop();
                    }
                    &opcodes::FRETURN => {
                        return stack.pop();
                    }
                    &opcodes::NEW => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let CpEntry::ClassRef(class_name_index) = method.constant_pool.get(&cp_index).unwrap() {
                           if let CpEntry::Utf8(class) = method.constant_pool.get(class_name_index).unwrap(){

                           }
                        }
                    }
                    //TODO implement all opcodes
                    _ => { panic!("opcode not implemented") }
                }
            }
        }
        None // TODO error situation
    }
}