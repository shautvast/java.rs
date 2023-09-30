use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Error};

use crate::class::{AttributeType, Class, Value};
use crate::class::Value::Void;
use crate::classloader::{load_class, CpEntry};
use crate::heap::{Heap, Object};
use crate::io::*;
use crate::opcodes::*;

#[derive(Debug)]
struct StackFrame {
    data: Vec<Arc<Value>>,
}

impl StackFrame {
    fn new() -> Self {
        Self { data: vec![] }
    }

    fn push(&mut self, val: Arc<Value>) {
        self.data.push(val);
    }

    fn pop(&mut self) -> Result<Arc<Value>, Error> {
        Ok(self.data.pop().unwrap())
    }
}

/// single threaded vm
pub struct Vm {
    classpath: Vec<String>,
    classes: HashMap<String, Arc<Class>>,
    heap: Heap,
    stack: Vec<StackFrame>,
}

const CP_SEP: char = ':';
//TODO semicolon on windows

impl Vm {
    fn local_stack(&mut self) -> &mut StackFrame {
        let i = self.stack.len() - 1;
        self.stack.get_mut(i).unwrap()
    }

    pub fn new(classpath: &'static str) -> Self {
        Self {
            classpath: classpath.split(CP_SEP).map(|s| s.to_owned()).collect(),
            classes: HashMap::new(),
            heap: Heap::new(),
            stack: vec![],
        }
    }

    /// parse the binary data into a Class struct
    /// gets the file from cache, or reads it from classpath
    /// Vm keeps ownership of the class and hands out Arc references to it
    pub fn get_class(&mut self, class_name: &str) -> Result<Arc<Class>, Error> {
        println!("get_class {}", class_name);
        let entry = self.classes.entry(class_name.into());
        let entry = entry.or_insert_with(|| {
            // print!("read class {} ", class_name);
            let resolved_path = find_class(&self.classpath, class_name).expect("Class not found");
            // println!("full path {}", resolved_path);
            let bytecode = read_bytecode(resolved_path).unwrap();
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
                _ => Value::Void,
            };
            data.insert(f.name_index, Arc::new(value));
        }
        Object::new(class.clone(), data)
    }

    /// execute the bytecode
    pub fn execute(
        &mut self,
        class_name: &str,
        method_name: &str,
        instance: Option<Arc<Value>>,
    ) -> Result<Arc<Value>, Error> {
        println!("execute {}.{}", class_name, method_name);
        let class = self.get_class(class_name)?;
        let method = class.get_method(method_name)?;
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let stackframe = StackFrame::new();
            self.stack.push(stackframe);

            let mut pc: usize = 0;
            while pc < code.opcodes.len() {
                let opcode = &code.opcodes[pc];
                pc += 1;
                println!("opcode {} ", opcode);
                match opcode {
                    BIPUSH => {
                        let c = code.opcodes[pc] as i32;
                        self.local_stack().push(Arc::new(Value::I32(c)));
                        pc += 1;
                    }
                    LDC => {
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.local_stack().push(Arc::new(Value::I32(*i)));
                            }
                            CpEntry::Float(f) => {
                                self.local_stack().push(Arc::new(Value::F32(*f)));
                            }
                            _ => {}
                        }
                        pc += 1;
                    }
                    LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.local_stack().push(Arc::new(Value::I32(*i)));
                            }
                            CpEntry::Float(f) => {
                                self.local_stack().push(Arc::new(Value::F32(*f)));
                            }
                            _ => {
                                panic!("unexpected")
                            }
                        }
                        pc += 2;
                    }
                    LDC2_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Double(d) => {
                                self.local_stack().push(Arc::new(Value::F64(*d)));
                            }
                            CpEntry::Long(l) => {
                                self.local_stack().push(Arc::new(Value::I64(*l)));
                            }
                            _ => {
                                panic!("unexpected")
                            }
                        }

                        pc += 2;
                    }
                    ALOAD_0 => {
                        println!("ALOAD_0");
                        match instance.clone() {
                            Some(instance) => {
                                self.local_stack().push(instance);
                            }
                            None => {
                                panic!("static context")
                            }
                        }
                    }
                    POP =>{
                        self.local_stack().pop().expect("Stack empty");
                    }
                    DUP => {
                        println!("DUP");
                        let value = self.local_stack().pop().expect("Stack empty");
                        println!("{:?}", value);
                        self.local_stack().push(value.clone());
                        self.local_stack().push(value);
                    }
                    IRETURN => {
                        println!("return I");
                        return self.local_stack().pop();
                    }
                    DRETURN => {
                        println!("return D");
                        return self.local_stack().pop();
                    }
                    FRETURN => {
                        println!("return F");
                        return self.local_stack().pop();
                    }
                    RETURN_VOID => {
                        println!("return");
                        self.stack.pop();
                        return Ok(Arc::new(Void));
                    }
                    GETFIELD => {
                        println!("GETFIELD");
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let CpEntry::Fieldref(_class_index, name_and_type_index) =
                            method.constant_pool.get(&cp_index).unwrap()
                        {
                            if let Value::Ref(inst) = &*self.local_stack().pop()? {
                                //TODO smell?
                                if let CpEntry::NameAndType(name, _) =
                                    method.constant_pool.get(name_and_type_index).unwrap()
                                {
                                    let value = inst.data.get(name).unwrap();
                                    println!("{:?}", value);
                                    self.local_stack().push(value.clone());
                                }
                            }
                        }
                        pc += 2;
                    }
                    INVOKEVIRTUAL => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        let instance = self.local_stack().pop().unwrap();
                        if let Some((class, method)) = get_signature_for_invoke(Rc::clone(&method.constant_pool), cp_index) {
                            let return_value = self.execute(class.as_str(), method.as_str(), Some(instance))?;
                            if let Void = *return_value {} else { // not let?
                                self.local_stack().push(return_value);
                            }
                        }

                        pc += 2;
                    }
                    INVOKESPECIAL => {
                        println!("INVOKESPECIAL");
                        let cp_index = read_u16(&code.opcodes, pc);
                        let instance = self.local_stack().pop().unwrap();
                        if let Some((class, method)) = get_signature_for_invoke(Rc::clone(&method.constant_pool), cp_index) {
                            let return_value = self.execute(class.as_str(), method.as_str(), Some(instance))?;
                            if let Void = *return_value {} else { // not let?
                                self.local_stack().push(return_value);
                            }
                        }

                        pc += 2;
                    }
                    NEW => {
                        let class_index = read_u16(&code.opcodes, pc);
                        println!("cp_index {}", class_index);
                        if let CpEntry::ClassRef(class_name_index) =
                            method.constant_pool.get(&class_index).unwrap()
                        {
                            if let CpEntry::Utf8(new_class) =
                                method.constant_pool.get(class_name_index).unwrap()
                            {
                                println!("new {}", new_class);
                                let class = self.get_class(new_class)?;
                                let object = Arc::new(self.new_instance(class));
                                self.local_stack().push(Arc::new(Value::Ref(object.clone())));
                                self.heap.new_object(object);
                            }
                        }
                        pc += 2;
                    }
                    //TODO implement all opcodes
                    _ => {
                        panic!("opcode not implemented {:?}", self.stack)
                    }
                }
            }
        }
        Err(anyhow!("should not happen"))
    }
}


//TODO refs with lifetime
fn get_signature_for_invoke(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<(String, String)> {
    if let CpEntry::MethodRef(class_index, name_and_type_index) = cp.get(&index).unwrap() {
        if let Some(method_signature) = get_name_and_type(Rc::clone(&cp), *name_and_type_index) {
            if let CpEntry::ClassRef(class_name_index) = cp.get(&class_index).unwrap() {
                if let CpEntry::Utf8(class_name) = cp.get(&class_name_index).unwrap() {
                    return Some((class_name.into(), method_signature));
                }
            }
        }
    }
    None
}

fn get_name_and_type(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<(String)> {
    if let CpEntry::NameAndType(method_name_index, signature_index) = cp.get(&index).unwrap() {
        if let CpEntry::Utf8(method_name) = cp.get(&method_name_index).unwrap() {
            if let CpEntry::Utf8(signature) = cp.get(&signature_index).unwrap() {
                let mut method_signature: String = method_name.into();
                method_signature.push_str(signature);
                return Some(method_signature);
            }
        }
    }
    None
}