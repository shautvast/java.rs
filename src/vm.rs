use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Error};

use crate::class::{AttributeType, Class, Value};
use crate::classloader::{CpEntry, load_class};
use crate::heap::{Heap, Object};
use crate::io::*;
use crate::opcodes;

struct StackFrame {
    data: Vec<Arc<Value>>,
}

impl StackFrame {
    fn new() -> Self {
        Self {
            data: vec![]
        }
    }

    fn push(&mut self, val: Arc<Value>) {
        self.data.push(val);
    }

    fn pop(&mut self) -> Result<Arc<Value>, Error> {
        Ok(self.data.pop().unwrap())
    }
}

pub struct Vm {
    classpath: Vec<String>,
    classes: HashMap<String, Arc<Class>>,
    //TODO implement classloader
    heap: Heap,
}

impl Vm {
    pub fn new(classpath: &'static str) -> Self {
        Self {
            classpath: classpath.split(":").into_iter().map(|s| s.to_owned()).collect(),
            classes: HashMap::new(),
            heap: Heap::new(),
        }
    }

    pub fn get_class(&mut self, class_name: &str) -> Result<Arc<Class>, Error> {
        let entry = self.classes.entry(class_name.into());
        let entry = entry.or_insert_with(|| {
            let resolved_path = find_class(&self.classpath, class_name).unwrap();
            let bytecode = read_class_file(resolved_path).unwrap();
            Arc::new(load_class(bytecode).unwrap())
        });
        Ok(entry.clone())
    }


    pub fn new_instance(&self, class: Arc<Class>) -> Object {
        let mut data = HashMap::new();
        for f in &class.fields {
            let value = match f.type_of().as_str() {
                "Z" => Value::BOOL(false),
                "B" => Value::I32(0),
                "S" => Value::I32(0),
                "I" => Value::I32(0),
                "J" => Value::I64(0),
                "F" => Value::F32(0.0),
                "D" => Value::F64(0.0),
                "L" => Value::Null,
                _ => Value::Void
            };
            data.insert(f.name_index, Arc::new(value));
        }
        Object::new(class.clone(), data)
    }

    pub fn execute(&mut self, class_name: &str, method_name: &str, instance: Option<Arc<Object>>) -> Result<Arc<Value>, Error> {
        let class = self.get_class(class_name)?;
        let method = class.get_method(method_name);
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let mut stack = StackFrame::new();
            let mut pc: usize = 0;
            while pc < code.opcodes.len() {
                let opcode = &code.opcodes[pc];
                pc += 1;
                println!("{}", opcode);
                match opcode {
                    opcodes::BIPUSH => {
                        let c = code.opcodes[pc] as i32;
                        stack.push(Arc::new(Value::I32(c)));
                        pc += 1;
                    }
                    &opcodes::LDC => {
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                stack.push(Arc::new(Value::I32(*i)));
                            }
                            CpEntry::Float(f) => {
                                stack.push(Arc::new(Value::F32(*f)));
                            }
                            _ => {}
                        }
                        pc += 1;
                    }
                    &opcodes::LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                stack.push(Arc::new(Value::I32(*i)));
                            }
                            CpEntry::Float(f) => {
                                stack.push(Arc::new(Value::F32(*f)));
                            }
                            _ => { panic!("unexpected") }
                        }
                        pc += 2;
                    }
                    &opcodes::LDC2_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Double(d) => {
                                stack.push(Arc::new(Value::F64(*d)));
                            }
                            CpEntry::Long(l) => {
                                stack.push(Arc::new(Value::I64(*l)));
                            }
                            _ => { panic!("unexpected") }
                        }

                        pc += 2;
                    }
                    &opcodes::ALOAD_0 => {
                        match instance.clone() {
                            Some(r) => {
                                stack.push(Arc::new(Value::Ref(r)));
                            }
                            None => { panic!("static context") }
                        }
                    }
                    &opcodes::IRETURN => {
                        return stack.pop();
                    }
                    &opcodes::DRETURN => {
                        return stack.pop();
                    }
                    &opcodes::FRETURN => {
                        return stack.pop();
                    }
                    &opcodes::GETFIELD => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let CpEntry::Fieldref(class_index, name_and_type_index) = method.constant_pool.get(&cp_index).unwrap() {
                            if let Value::Ref(inst) = &*stack.pop()? { //TODO smell?
                                if let CpEntry::NameAndType(name, _) = method.constant_pool.get(name_and_type_index).unwrap() {
                                    let value = inst.data.get(&name).unwrap();
                                    // println!("{:?}", value);
                                    stack.push(value.clone());
                                }
                            }
                        }
                        pc += 2;
                    }
                    &opcodes::NEW => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let CpEntry::ClassRef(class_name_index) = method.constant_pool.get(&cp_index).unwrap() {
                            if let CpEntry::Utf8(class) = method.constant_pool.get(class_name_index).unwrap() {
                                let class = self.get_class(class_name)?;
                                let object = Arc::new(self.new_instance(class));
                                stack.push(Arc::new(Value::Ref(object.clone())));
                                self.heap.new_object(object);
                            }
                        }
                    }
                    //TODO implement all opcodes
                    _ => { panic!("opcode not implemented") }
                }
            }
        }
        Err(anyhow!("should not happen"))
    }
}