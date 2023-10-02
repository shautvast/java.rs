use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Error;

use crate::class::{AttributeType, Class, Value};
use crate::class::Value::Void;
use crate::classloader::{CpEntry, load_class};
use crate::heap::{Heap, Object};
use crate::io::*;
use crate::opcodes::*;

#[derive(Debug)]
struct StackFrame {
    at: String,
    data: Vec<Arc<UnsafeCell<Value>>>,
}

impl StackFrame {
    fn new(at_class: &str, at_method: &str) -> Self {
        let mut at: String = at_class.into();
        at.push('.');
        at.push_str(at_method);
        Self { at, data: vec![] }
    }

    fn push(&mut self, val: Arc<UnsafeCell<Value>>) {
        self.data.push(val);
    }

    fn pop(&mut self) -> Result<Arc<UnsafeCell<Value>>, Error> {
        Ok(self.data.pop().unwrap())
    }
}

/// single threaded vm
pub struct Vm {
    classpath: Vec<String>,
    classes: HashMap<String, Rc<Class>>,
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
    pub fn get_class(&mut self, class_name: &str) -> Result<Rc<Class>, Error> {
        println!("get_class {}", class_name);
        let entry = self.classes.entry(class_name.into());
        let entry = entry.or_insert_with(|| {
            // print!("read class {} ", class_name);
            let resolved_path = find_class(&self.classpath, class_name).expect("Class not found");
            // println!("full path {}", resolved_path);
            let bytecode = read_bytecode(resolved_path).unwrap();
            Rc::new(load_class(bytecode).unwrap())
        });
        Ok(entry.clone())
    }

    pub fn new_instance(&self, class: Rc<Class>) -> Object {
        //TODO add fields from superclasses
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
            data.insert(f.name_index, Arc::new(UnsafeCell::new(value)));
        }
        Object::new(class.clone(), data)
    }

    /// execute the bytecode
    pub fn execute(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: Vec<Arc<UnsafeCell<Value>>>,
    ) -> Result<Arc<UnsafeCell<Value>>, Error> {
        println!("execute {}.{}", class_name, method_name);
        let class = self.get_class(class_name)?;
        let method = class.get_method(method_name)?;
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let stackframe = StackFrame::new(class_name, method_name);
            self.stack.push(stackframe);

            let mut pc= &mut 0;
            while *pc < code.opcodes.len() {
                let opcode = read_u8(&code.opcodes, pc);
                println!("opcode {} ", opcode);
                match opcode {
                    BIPUSH => {
                        println!("BISPUSH");
                        let c =read_u8(&code.opcodes, pc);
                        self.local_stack().push(Arc::new(UnsafeCell::new(Value::I32(c as i32))));
                    }
                    LDC => {
                        println!("LDC");
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::I32(*i))));
                            }
                            CpEntry::Float(f) => {
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::F32(*f))));
                            }
                            _ => {}
                        }
                    }
                    LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::I32(*i))));
                            }
                            CpEntry::Float(f) => {
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::F32(*f))));
                            }
                            _ => {
                                panic!("unexpected")
                            }
                        }
                    }
                    LDC2_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Double(d) => {
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::F64(*d))));
                            }
                            CpEntry::Long(l) => {
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::I64(*l))));
                            }
                            _ => {
                                panic!("unexpected")
                            }
                        }

                    }
                    FLOAD_0 => {
                        self.local_stack().push(args[0].clone());
                    }
                    FLOAD_1 => {
                        println!("FLOAD_1");
                        self.local_stack().push(args[1].clone());
                    }
                    FLOAD_2 => {
                        self.local_stack().push(args[2].clone());
                    }
                    FLOAD_3 => {
                        self.local_stack().push(args[3].clone());
                    }
                    ALOAD_0 => {
                        println!("ALOAD_0");
                        self.local_stack().push(args[0].clone());
                    }
                    ALOAD_1 => {
                        println!("ALOAD_1");
                        self.local_stack().push(args[1].clone());
                    }
                    ALOAD_2 => {
                        println!("ALOAD_2");
                        self.local_stack().push(args[2].clone());
                    }
                    ALOAD_3 => {
                        println!("ALOAD_3");
                        self.local_stack().push(args[3].clone());
                    }
                    POP => {
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
                        return Ok(Arc::new(UnsafeCell::new(Void)));
                    }
                    GETFIELD => {
                        println!("GETFIELD");
                        unsafe {
                            let cp_index = read_u16(&code.opcodes, pc);
                            if let CpEntry::Fieldref(_class_index, name_and_type_index) =
                                method.constant_pool.get(&cp_index).unwrap()
                            {
                                if let Value::Ref(instance) = &*self.local_stack().pop()?.get() {
                                    if let CpEntry::NameAndType(name, _) =
                                        method.constant_pool.get(name_and_type_index).unwrap()
                                    {
                                        let value = (*(*instance).get()).data.get(name).unwrap();
                                        self.local_stack().push(Arc::clone(value));
                                    }
                                }
                            }
                        }
                    }
                    PUTFIELD => {
                        println!("PUTFIELD");
                        unsafe {
                            let cp_index = read_u16(&code.opcodes, pc);
                            if let CpEntry::Fieldref(_class_index, name_and_type_index) =
                                method.constant_pool.get(&cp_index).unwrap()
                            {
                                if let CpEntry::NameAndType(name_index, _) = method.constant_pool.get(name_and_type_index).unwrap() {
                                    let value = self.local_stack().pop()?;
                                    let mut objectref = &*self.local_stack().pop()?.get();
                                    if let Value::Ref(instance) = objectref {
                                        (*(*instance).get()).data.insert(*name_index, value);
                                    }
                                }
                            }
                        }
                    }
                    INVOKEVIRTUAL => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        unsafe {
                            if let Some((class, method)) = get_signature_for_invoke(Rc::clone(&method.constant_pool), cp_index) {
                                let signature = method.0.as_str();
                                let num_args = method.1;
                                let mut args = Vec::with_capacity(num_args);
                                for _ in 0..num_args {
                                    args.insert(0, self.local_stack().pop()?);
                                }
                                args.insert(0, self.local_stack().pop()?);
                                let mut returnvalue = self.execute(class.as_str(), signature, args)?;
                                match *returnvalue.get() {
                                    Void => {}
                                    _ => { self.local_stack().push(returnvalue.clone()); }
                                }
                            }
                        }
                    }
                    INVOKESPECIAL => {
                        println!("INVOKESPECIAL");
                        unsafe {
                            let cp_index = read_u16(&code.opcodes, pc);
                            if let Some((class, method)) = get_signature_for_invoke(Rc::clone(&method.constant_pool), cp_index) {
                                let signature = method.0.as_str();
                                let num_args = method.1;
                                let mut args = vec![];
                                for _ in 0..num_args {
                                    args.insert(0, self.local_stack().pop()?);
                                }
                                args.insert(0, self.local_stack().pop()?);
                                let mut returnvalue = self.execute(class.as_str(), signature, args)?;
                                match *returnvalue.get() {
                                    Void => {}
                                    _ => { self.local_stack().push(returnvalue.clone()); }
                                }
                            }
                        }
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
                                let object = Arc::new(UnsafeCell::new(self.new_instance(class)));
                                self.local_stack().push(Arc::new(UnsafeCell::new(Value::Ref(Arc::clone(&object)))));
                                self.heap.new_object(object);
                            }
                        }
                    }
                    //TODO implement all opcodes
                    _ => {
                        panic!("opcode not implemented {:?}", self.stack)
                        //TODO implement proper stacktraces
                    }
                }
            }
        }
        panic!("should not happen")
    }
}


//TODO refs with lifetime
fn get_signature_for_invoke(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<(String, (String, usize))> {
    if let CpEntry::MethodRef(class_index, name_and_type_index) = cp.get(&index).unwrap() {
        if let Some(method_signature) = get_name_and_type(Rc::clone(&cp), *name_and_type_index) {
            if let CpEntry::ClassRef(class_name_index) = cp.get(class_index).unwrap() {
                if let CpEntry::Utf8(class_name) = cp.get(class_name_index).unwrap() {
                    return Some((class_name.into(), method_signature));
                }
            }
        }
    }
    None
}

fn get_name_and_type(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<(String, usize)> {
    if let CpEntry::NameAndType(method_name_index, signature_index) = cp.get(&index).unwrap() {
        if let CpEntry::Utf8(method_name) = cp.get(method_name_index).unwrap() {
            if let CpEntry::Utf8(signature) = cp.get(signature_index).unwrap() {
                let mut method_signature: String = method_name.into();
                let num_args = get_hum_args(signature);
                method_signature.push_str(signature);

                return Some((method_signature, num_args));
            }
        }
    }
    None
}

fn get_hum_args(signature: &str) -> usize {
    let mut num = 0;
    let mut i = 1;
    let chars: Vec<char> = signature.chars().collect();

    while i < chars.len() {
        if chars[i] == 'L' {
            i += 1;
            while chars[i] != ';' {
                i += 1;
            }
            num += 1;
        } else if chars[i] == ')' {
            break;
        } else {
            i += 1;
            num += 1;
        }
    }
    num
}