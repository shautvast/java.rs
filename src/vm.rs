use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Error};
use once_cell::unsync::Lazy;

use crate::class::{AttributeType, Class, Value};
use crate::class::Value::Void;
use crate::classloader::{CpEntry, load_class};
use crate::heap::{Heap, Object, ObjectRef};
use crate::io::*;
use crate::opcodes::*;

#[derive(Debug)]
struct StackFrame {
    at: String,
    data: Vec<Arc<UnsafeCell<Value>>>,
}

// maybe just call frame
impl StackFrame {
    fn new(at_class: &str, at_method: &str) -> Self {
        let mut at: String = at_class.into();
        at.push('.');
        at.push_str(at_method);
        Self { at, data: vec![] }
    }

    fn push(&mut self, val: Value) {
        self.data.push(Arc::new(UnsafeCell::new(val)));
    }

    fn push_arc(&mut self, val: Arc<UnsafeCell<Value>>) {
        self.data.push(val);
    }

    fn pop(&mut self) -> Result<Arc<UnsafeCell<Value>>, Error> {
        Ok(self.data.pop().unwrap())
    }
}

//trying to be ready for multithreaded as much as possible, using Arc's and all, but it will still require (a lot of) extra work
static mut CLASSDEFS: Lazy<HashMap<String, Rc<Class>>> = Lazy::new(|| HashMap::new()); //TODO add mutex...and Arc most likely

pub struct Vm {
    classpath: Vec<String>,
    heap: Heap,
    stack: Vec<StackFrame>,
}

#[cfg(target_family = "unix")]
const PATH_SEPARATOR: char = ':';

#[cfg(target_family = "windows")]
const PATH_SEPARATOR: char = ';';

// The singlethreaded VM (maybe a future Thread)
impl Vm {
    fn local_stack(&mut self) -> &mut StackFrame {
        let i = self.stack.len() - 1;
        self.stack.get_mut(i).unwrap()
    }

    pub fn new(classpath: &'static str) -> Self {
        Self {
            classpath: classpath.split(PATH_SEPARATOR).map(|s| s.to_owned()).collect(),
            heap: Heap::new(),
            stack: vec![],
        }
    }

    // parse the binary data into a Class struct
    // gets the file from cache, or reads it from classpath
    // Vm keeps ownership of the class and hands out Arc references to it
    pub fn get_class(&self, class_name: &str) -> Result<Rc<Class>, Error> {
        println!("get_class {}", class_name);
        unsafe {
            let entry = CLASSDEFS.entry(class_name.into());
            let entry = entry.or_insert_with(|| {
                // print!("read class {} ", class_name);
                let resolved_path = find_class(&self.classpath, class_name).unwrap();
                // println!("full path {}", resolved_path);
                let bytecode = read_bytecode(resolved_path).unwrap();
                let mut class = load_class(bytecode).unwrap();
                let super_class_name = class.super_class_name.as_ref();
                if let Some(super_class_name) = super_class_name {
                    if let Ok(super_class) = self.get_class(&super_class_name) {
                        class.super_class = Some(super_class.clone());
                    }
                }
                class.initialize();

                Rc::new(class)
            });
            Ok(entry.clone())
        }
    }

    pub fn new_instance(class: Rc<Class>) -> Object {
        let mut class = class;
        // let mut outer = HashMap::new();
        //TODO

        let mut instance = Object::new(class.clone());
        instance.init_fields();
        instance
    }


    /// execute the bytecode
    /// contains unsafe, as I think that mimics not-synchronized memory access in the original JVM
    pub fn execute(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: Vec<Arc<UnsafeCell<Value>>>,
    ) -> Result<Arc<UnsafeCell<Value>>, Error> {
        let mut local_params: Vec<Option<Arc<UnsafeCell<Value>>>> = args.clone().iter().map(|e| Some(e.clone())).collect();
        println!("execute {}.{}", class_name, method_name);
        let class = self.get_class(class_name)?;
        let method = class.get_method(method_name)?;
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let stackframe = StackFrame::new(class_name, method_name);
            self.stack.push(stackframe);

            let mut pc = &mut 0;
            while *pc < code.opcodes.len() {
                let opcode = read_u8(&code.opcodes, pc);
                println!("opcode {} ", opcode);
                match opcode {
                    ACONST_NULL => {
                        self.local_stack().push(Value::Null);
                    }
                    ICONST_M1 => {
                        self.local_stack().push(Value::I32(-1));
                    }
                    ICONST_0 => {
                        self.local_stack().push(Value::I32(0));
                    }
                    ICONST_1 => {
                        self.local_stack().push(Value::I32(1));
                    }
                    ICONST_2 => {
                        self.local_stack().push(Value::I32(2));
                    }
                    ICONST_3 => {
                        self.local_stack().push(Value::I32(3));
                    }
                    ICONST_4 => {
                        self.local_stack().push(Value::I32(4));
                    }
                    ICONST_5 => {
                        self.local_stack().push(Value::I32(5));
                    }
                    LCONST_0 => {
                        self.local_stack().push(Value::I64(0));
                    }
                    LCONST_1 => {
                        self.local_stack().push(Value::I64(1));
                    }
                    FCONST_0 => {
                        self.local_stack().push(Value::F32(0.0));
                    }
                    FCONST_1 => {
                        self.local_stack().push(Value::F32(1.0));
                    }
                    FCONST_2 => {
                        self.local_stack().push(Value::F32(2.0));
                    }
                    DCONST_0 => {
                        self.local_stack().push(Value::F64(0.0));
                    }
                    DCONST_1 => {
                        self.local_stack().push(Value::F64(1.0));
                    }
                    SIPUSH => {
                        let s = read_u16(&code.opcodes, pc) as i32;
                        self.local_stack().push(Value::I32(s));
                    }
                    BIPUSH => {
                        let c = read_u8(&code.opcodes, pc) as i32;
                        self.local_stack().push(Value::I32(c));
                    }
                    LDC => {
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.local_stack().push(Value::I32(*i));
                            }
                            CpEntry::Float(f) => {
                                self.local_stack().push(Value::F32(*f));
                            }
                            _ => {}
                        }
                    }
                    LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.local_stack().push(Value::I32(*i));
                            }
                            CpEntry::Float(f) => {
                                self.local_stack().push(Value::F32(*f));
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
                                self.local_stack().push(Value::F64(*d));
                            }
                            CpEntry::Long(l) => {
                                self.local_stack().push(Value::I64(*l));
                            }
                            _ => {
                                panic!("unexpected")
                            }
                        }
                    }
                    ILOAD | LLOAD | FLOAD | DLOAD | ALOAD => { // omitting the type checks so far
                        let n = read_u8(&code.opcodes, pc) as usize;
                        self.local_stack().push_arc(local_params[n].as_ref().unwrap().clone());
                    }
                    ILOAD_0 | LLOAD_0 | FLOAD_0 | DLOAD_0 | ALOAD_0 => {
                        self.local_stack().push_arc(local_params[0].as_ref().unwrap().clone());
                    }
                    ILOAD_1 | LLOAD_1 | FLOAD_1 | DLOAD_1 | ALOAD_1 => {
                        self.local_stack().push_arc(local_params[1].as_ref().unwrap().clone());
                    }
                    ILOAD_2 | LLOAD_2 | FLOAD_2 | DLOAD_2 | ALOAD_2 => {
                        self.local_stack().push_arc(local_params[2].as_ref().unwrap().clone());
                    }
                    ILOAD_3 | LLOAD_3 | FLOAD_3 | DLOAD_3 | ALOAD_3 => {
                        self.local_stack().push_arc(local_params[3].as_ref().unwrap().clone());
                    }
                    IALOAD | LALOAD | FALOAD | DALOAD | AALOAD | BALOAD | CALOAD | SALOAD => unsafe {
                        self.array_load()?;
                    }
                    ISTORE | LSTORE | FSTORE | DSTORE | ASTORE => {
                        let index = read_u8(&code.opcodes, pc) as usize;
                        self.store(&mut local_params, index)?;
                    }
                    ISTORE_0 | LSTORE_0 | DSTORE_0 | ASTORE_0 | FSTORE_0 => {
                        self.store(&mut local_params, 0)?;
                    }
                    ISTORE_1 | LSTORE_1 | DSTORE_1 | ASTORE_1 | FSTORE_1 => {
                        self.store(&mut local_params, 1)?;
                    }
                    ISTORE_2 | LSTORE_2 | DSTORE_2 | ASTORE_2 | FSTORE_2 => {
                        self.store(&mut local_params, 2)?;
                    }
                    ISTORE_3 | LSTORE_3 | DSTORE_3 | ASTORE_3 | FSTORE_3 => {
                        self.store(&mut local_params, 3)?;
                    }
                    BASTORE | IASTORE | LASTORE | CASTORE | SASTORE | FASTORE | DASTORE | AASTORE => unsafe { self.array_store()? }
                    POP => {
                        self.local_stack().pop()?;
                    }
                    DUP => {
                        let value = self.local_stack().pop()?;
                        println!("{:?}", value);
                        self.local_stack().push_arc(value.clone());
                        self.local_stack().push_arc(value);
                    }
                    IRETURN | FRETURN | DRETURN => {
                        return self.local_stack().pop();
                    }
                    RETURN_VOID => {
                        self.stack.pop(); // Void is also returned as a value
                        return Ok(Arc::new(UnsafeCell::new(Void)));
                    }
                    GETSTATIC => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, _field_name_and_type_index) = class.get_field_ref(&cp_index).unwrap();
                        let class_name_index = class.get_class_ref(class_index).unwrap();
                        let class_name = class.get_utf8(class_name_index).unwrap();
                        let class = self.get_class(class_name.as_str())?;
                        println!("{:?}", class); //TODO
                    }
                    GETFIELD => {
                        unsafe {
                            let cp_index = read_u16(&code.opcodes, pc);
                            let (class_index, field_name_and_type_index) = class.get_field_ref(&cp_index).unwrap();
                            let (field_name_index, _) = class.get_name_and_type(field_name_and_type_index).unwrap();
                            let class_name_index = class.get_class_ref(class_index).unwrap();
                            let class_name = class.get_utf8(class_name_index).unwrap();
                            let field_name = class.get_utf8(field_name_index).unwrap();

                            let mut objectref = self.local_stack().pop()?;
                            if let Value::Ref(instance) = &mut *objectref.get() {
                                if let ObjectRef::Object(ref mut object) = &mut *instance.get() {
                                    let value = object.get(class_name, field_name);
                                    self.local_stack().push_arc(Arc::clone(value));
                                }
                            }
                        }
                    }
                    PUTFIELD => unsafe {
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) = class.get_field_ref(&cp_index).unwrap();
                        let (field_name_index, _) = class.get_name_and_type(field_name_and_type_index).unwrap();
                        let class_name_index = class.get_class_ref(class_index).unwrap();
                        let class_name = class.get_utf8(class_name_index).unwrap();
                        let field_name = class.get_utf8(field_name_index).unwrap();

                        let value = self.local_stack().pop()?;
                        let mut objectref = self.local_stack().pop()?;
                        if let Value::Ref(instance) = &mut *objectref.get() {
                            if let ObjectRef::Object(ref mut object) = &mut *instance.get() {
                                object.set(class_name, field_name, value);
                            }
                        }
                    }
                    INVOKEVIRTUAL => unsafe {
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) = get_signature_for_invoke(Rc::clone(&method.constant_pool), cp_index) {
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, self.local_stack().pop()?);
                            }
                            args.insert(0, self.local_stack().pop()?);
                            let mut returnvalue = self.execute(&invocation.class_name, &invocation.method.name, args)?;
                            match *returnvalue.get() {
                                Void => {}
                                _ => { self.local_stack().push_arc(returnvalue.clone()); }
                            }
                        }
                    }
                    INVOKESPECIAL => unsafe {
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) = get_signature_for_invoke(Rc::clone(&method.constant_pool), cp_index) {
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, self.local_stack().pop()?);
                            }
                            args.insert(0, self.local_stack().pop()?);
                            let mut returnvalue = self.execute(&invocation.class_name, &invocation.method.name, args)?;
                            match *returnvalue.get() {
                                Void => {}
                                _ => { self.local_stack().push_arc(returnvalue.clone()); }
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
                                let object = Arc::new(UnsafeCell::new(ObjectRef::Object(Box::new(Vm::new_instance(class)))));
                                self.local_stack().push(Value::Ref(Arc::clone(&object)));
                                self.heap.new_object(object);
                            }
                        }
                    }
                    //TODO implement all opcodes
                    _ => {
                        panic!("opcode not implemented {:?}", self.stack)
                        //TODO implement proper --stacktraces-- error handling
                    }
                }
            }
        }
        panic!("should not happen")
    }

    unsafe fn array_load(&mut self) -> Result<(), Error> {
        if let Value::I32(index) = &*self.local_stack().pop()?.get() {
            let index = *index as usize;
            let arrayref = &*self.local_stack().pop()?.get();
            if let Value::Null = arrayref {
                return Err(anyhow!("NullpointerException"));
            }
            if let Value::Ref(ref objectref) = arrayref {
                match &*objectref.get() {
                    ObjectRef::ByteArray(ref array) => {
                        self.local_stack().push(Value::I32(array[index] as i32));
                    }
                    ObjectRef::ShortArray(ref array) => {
                        self.local_stack().push(Value::I32(array[index] as i32));
                    }
                    ObjectRef::IntArray(ref array) => {
                        self.local_stack().push(Value::I32(array[index]));
                    }
                    ObjectRef::BooleanArray(ref array) => {
                        self.local_stack().push(Value::I32(array[index] as i32));
                    }
                    ObjectRef::CharArray(ref array) => {
                        self.local_stack().push(Value::CHAR(array[index]));
                    }
                    ObjectRef::LongArray(ref array) => {
                        self.local_stack().push(Value::I64(array[index]));
                    }
                    ObjectRef::FloatArray(ref array) => {
                        self.local_stack().push(Value::F32(array[index]));
                    }
                    ObjectRef::DoubleArray(ref array) => {
                        self.local_stack().push(Value::F64(array[index]));
                    }
                    ObjectRef::ObjectArray(ref array) => {
                        self.local_stack().push(Value::Ref(array.get(index).unwrap().clone()));
                    }
                    ObjectRef::Object(_) => {} //throw error?
                }
            }
        }
        Ok(())
    }

    unsafe fn array_store(&mut self) -> Result<(), Error> {
        let value = self.local_stack().pop()?;
        let index = &mut *self.local_stack().pop()?.get();
        let mut arrayref = &mut *self.local_stack().pop()?.get();

        if let Value::Null = arrayref {
            return Err(anyhow!("NullpointerException"));
        }

        if let Value::I32(index) = index {
            if let Value::Ref(ref mut objectref) = arrayref {
                match &mut *objectref.get() {
                    ObjectRef::ByteArray(ref mut array) => {
                        if let Value::I32(value) = *value.get() { // is i32 correct?
                            array[*index as usize] = value as i8;
                        }
                    }
                    ObjectRef::ShortArray(ref mut array) => {
                        if let Value::I32(value) = *value.get() { // is i32 correct?
                            array[*index as usize] = value as i16;
                        }
                    }
                    ObjectRef::IntArray(ref mut array) => {
                        if let Value::I32(value) = *value.get() {
                            array[*index as usize] = value;
                        }
                    }
                    ObjectRef::BooleanArray(ref mut array) => {
                        if let Value::I32(value) = *value.get() {
                            array[*index as usize] = value > 0;
                        }
                    }
                    ObjectRef::CharArray(ref mut array) => {
                        if let Value::I32(value) = *value.get() {
                            array[*index as usize] = char::from_u32_unchecked(value as u32);
                        }
                    }
                    ObjectRef::LongArray(ref mut array) => {
                        if let Value::I64(value) = *value.get() {
                            array[*index as usize] = value;
                        }
                    }
                    ObjectRef::FloatArray(ref mut array) => {
                        if let Value::F32(value) = *value.get() {
                            array[*index as usize] = value
                        }
                    }
                    ObjectRef::DoubleArray(ref mut array) => {
                        if let Value::F64(value) = *value.get() {
                            array[*index as usize] = value
                        }
                    }
                    ObjectRef::ObjectArray(ref mut array) => {
                        if let Value::Ref(ref value) = *value.get() {
                            array[*index as usize] = value.clone();
                        }
                    }
                    ObjectRef::Object(_) => {}//throw error?
                }
            }
        }
        Ok(())
    }

    fn store(&mut self, local_params: &mut Vec<Option<Arc<UnsafeCell<Value>>>>, index: usize) -> Result<(), Error> {
        let value = self.local_stack().pop()?;
        while local_params.len() < index + 1 {
            local_params.push(None);
        }
        local_params[index] = Some(value.clone());
        Ok(())
    }
}


struct Invocation {
    class_name: String,
    method: MethodSignature,
}

struct MethodSignature {
    name: String,
    num_args: usize,
}

fn get_signature_for_invoke(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<Invocation> {
    if let CpEntry::MethodRef(class_index, name_and_type_index) = cp.get(&index).unwrap() {
        if let Some(method_signature) = get_name_and_type(Rc::clone(&cp), *name_and_type_index) {
            if let CpEntry::ClassRef(class_name_index) = cp.get(class_index).unwrap() {
                if let CpEntry::Utf8(class_name) = cp.get(class_name_index).unwrap() {
                    return Some(Invocation {
                        class_name: class_name.into(),
                        method: method_signature,
                    });
                }
            }
        }
    }
    None
}

fn get_name_and_type(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<MethodSignature> {
    if let CpEntry::NameAndType(method_name_index, signature_index) = cp.get(&index).unwrap() {
        if let CpEntry::Utf8(method_name) = cp.get(method_name_index).unwrap() {
            if let CpEntry::Utf8(signature) = cp.get(signature_index).unwrap() {
                let mut method_signature: String = method_name.into();
                let num_args = get_hum_args(signature);
                method_signature.push_str(signature);
                return Some(MethodSignature { name: method_signature, num_args });
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