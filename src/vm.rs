use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Error};

use Value::{*};

use crate::class::{AttributeType, Class, get_class, Modifier, unsafe_ref, unsafe_val, UnsafeValue, Value};
use crate::class::Value::{Null, Void};
use crate::classloader::CpEntry;
use crate::heap::{Heap, Object, ObjectRef};
use crate::io::*;
use crate::native::invoke_native;
use crate::opcodes::*;

#[derive(Debug)]
pub(crate) struct StackFrame {
    at: String,
    data: Vec<UnsafeValue>,
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
        self.data.push(unsafe_val(val));
    }

    fn push_ref(&mut self, val: UnsafeValue) {
        self.data.push(val);
    }

    fn pop(&mut self) -> Result<UnsafeValue, Error> {
        Ok(self.data.pop().unwrap())
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}


pub struct Vm {
    pub classpath: Vec<String>,
    heap: Heap,
    pub(crate) stackframes: Vec<StackFrame>,
}

#[cfg(target_family = "unix")]
const PATH_SEPARATOR: char = ':';

#[cfg(target_family = "windows")]
const PATH_SEPARATOR: char = ';';

/// The singlethreaded VM (maybe a future Thread)
//TODO goto
//TODO error handling
impl Vm {
    fn current_frame(&mut self) -> &mut StackFrame {
        let i = self.stackframes.len() - 1;
        self.stackframes.get_mut(i).unwrap()
    }

    pub fn new(classpath: &'static str) -> Self {
        Self {
            classpath: classpath
                .split(PATH_SEPARATOR)
                .map(|s| s.to_owned())
                .collect(),
            heap: Heap::new(),
            stackframes: vec![],
        }
    }


    pub fn new_instance(class: Arc<RefCell<Class>>) -> Object {
        let mut instance = Object::new(class.clone());
        instance
    }

    /// execute the bytecode
    /// contains unsafe, as I think that mimics not-synchronized memory access in the original JVM
    pub fn execute(
        &mut self,
        calling_class_name: Option<&str>,
        class_name: &str,
        method_name: &str,
        args: Vec<UnsafeValue>,
    ) -> Result<UnsafeValue, Error> {
        let class = get_class(self, calling_class_name, class_name)?;
        self.execute_class(class, method_name, args)
    }

    pub fn execute_class(
        &mut self,
        class: Arc<RefCell<Class>>,
        method_name: &str,
        args: Vec<UnsafeValue>,
    ) -> Result<UnsafeValue, Error> {
        let this_class = class;
        println!("execute {}.{}", this_class.borrow().name, method_name);

        let method = this_class.clone().borrow().get_method(method_name)?.clone();
        let mut local_params: Vec<Option<UnsafeValue>> =
            args.clone().iter().map(|e| Some(e.clone())).collect();
        if method.is(Modifier::Native) {
            return Ok(invoke_native(method, args));
        }
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let stackframe = StackFrame::new(&this_class.borrow().name, &method.name());
            self.stackframes.push(stackframe);

            let mut pc = &mut 0;
            while *pc < code.opcodes.len() {
                let opcode = read_u8(&code.opcodes, pc);
                let f = self.current_frame();
                print!("at {}: stack {}, #{}: opcode {}: ", f.at, f.len(), *pc-1, opcode);
                match opcode {
                    ACONST_NULL => {
                        println!("ACONST");
                        self.current_frame().push(Value::Null);
                    }
                    ICONST_M1 => {
                        println!("ICONST");
                        self.current_frame().push(I32(-1));
                    }
                    ICONST_0 => {
                        println!("ICONST");
                        self.current_frame().push(I32(0));
                    }
                    ICONST_1 => {
                        println!("ICONST");
                        self.current_frame().push(I32(1));
                    }
                    ICONST_2 => {
                        println!("ICONST");
                        self.current_frame().push(I32(2));
                    }
                    ICONST_3 => {
                        println!("ICONST");
                        self.current_frame().push(I32(3));
                    }
                    ICONST_4 => {
                        println!("ICONST");
                        self.current_frame().push(I32(4));
                    }
                    ICONST_5 => {
                        println!("ICONST");
                        self.current_frame().push(I32(5));
                    }
                    LCONST_0 => {
                        println!("LCONST");
                        self.current_frame().push(Value::I64(0));
                    }
                    LCONST_1 => {
                        println!("LCONST");
                        self.current_frame().push(Value::I64(1));
                    }
                    FCONST_0 => {
                        println!("FCONST");
                        self.current_frame().push(Value::F32(0.0));
                    }
                    FCONST_1 => {
                        println!("FCONST");
                        self.current_frame().push(Value::F32(1.0));
                    }
                    FCONST_2 => {
                        println!("FCONST");
                        self.current_frame().push(Value::F32(2.0));
                    }
                    DCONST_0 => {
                        println!("DCONST");
                        self.current_frame().push(Value::F64(0.0));
                    }
                    DCONST_1 => {
                        println!("DCONST");
                        self.current_frame().push(Value::F64(1.0));
                    }
                    SIPUSH => {
                        println!("SIPUSH");
                        let s = read_u16(&code.opcodes, pc) as i32;
                        self.current_frame().push(I32(s));
                    }
                    BIPUSH => {
                        println!("BIPUSH");
                        let c = read_u8(&code.opcodes, pc) as i32;
                        self.current_frame().push(I32(c));
                    }
                    LDC => {
                        println!("LDC");
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        let c = method.constant_pool.get(&cp_index).unwrap();
                        println!("{:?}", c);
                        match c {
                            CpEntry::Integer(i) => {
                                self.current_frame().push(I32(*i));
                            }
                            CpEntry::Float(f) => {
                                self.current_frame().push(Value::F32(*f));
                            }
                            CpEntry::Double(d) => {
                                self.current_frame().push(Value::F64(*d));
                            }
                            CpEntry::StringRef(utf8) => {
                                //TODO
                                let stringclass = get_class(self, Some(&this_class.borrow().name), "java/lang/String").unwrap();
                                let stringinstance = unsafe_val(
                                    Value::Ref(unsafe_ref(ObjectRef::Object(Box::new(
                                        Vm::new_instance(stringclass.clone()))))));
                                let string: Vec<u8> = this_class.borrow().cp_utf8(utf8).unwrap().to_owned().as_bytes().into();

                                self.execute_class(stringclass, "<init>([B)V",
                                                   vec![stringinstance.clone(),
                                                        unsafe_val(Value::Ref(ObjectRef::new_byte_array(string)))])?;
                                self.current_frame().push_ref(stringinstance);
                            }
                            CpEntry::Long(l) => {
                                self.current_frame().push(Value::I64(*l));
                            }
                            _ => {}
                        }
                    }
                    LDC_W => {
                        println!("LDC_W");
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Integer(i) => {
                                self.current_frame().push(I32(*i));
                            }
                            CpEntry::Float(f) => {
                                self.current_frame().push(Value::F32(*f));
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    LDC2_W => {
                        println!("LDC2_W");
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Double(d) => {
                                self.current_frame().push(Value::F64(*d));
                            }
                            CpEntry::Long(l) => {
                                self.current_frame().push(Value::I64(*l));
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    ILOAD | LLOAD | FLOAD | DLOAD | ALOAD => {
                        println!("LOAD");
                        // omitting the type checks so far
                        let n = read_u8(&code.opcodes, pc) as usize;
                        self.current_frame()
                            .push_ref(local_params[n].as_ref().unwrap().clone());
                    }
                    ILOAD_0 | LLOAD_0 | FLOAD_0 | DLOAD_0 | ALOAD_0 => {
                        println!("LOAD_0");
                        self.current_frame()
                            .push_ref(local_params[0].as_ref().unwrap().clone());
                    }
                    ILOAD_1 | LLOAD_1 | FLOAD_1 | DLOAD_1 | ALOAD_1 => {
                        println!("LOAD_1");
                        self.current_frame()
                            .push_ref(local_params[1].as_ref().unwrap().clone());
                    }
                    ILOAD_2 | LLOAD_2 | FLOAD_2 | DLOAD_2 | ALOAD_2 => {
                        println!("LOAD_2");
                        self.current_frame()
                            .push_ref(local_params[2].as_ref().unwrap().clone());
                    }
                    ILOAD_3 | LLOAD_3 | FLOAD_3 | DLOAD_3 | ALOAD_3 => {
                        println!("LOAD_3");
                        self.current_frame()
                            .push_ref(local_params[3].as_ref().unwrap().clone());
                    }
                    IALOAD | LALOAD | FALOAD | DALOAD | AALOAD | BALOAD | CALOAD | SALOAD => unsafe {
                        println!("ALOAD");
                        self.array_load()?;
                    },
                    ISTORE | LSTORE | FSTORE | DSTORE | ASTORE => {
                        println!("STORE");
                        let index = read_u8(&code.opcodes, pc) as usize;
                        self.store(&mut local_params, index)?;
                    }
                    ISTORE_0 | LSTORE_0 | DSTORE_0 | ASTORE_0 | FSTORE_0 => {
                        println!("STORE_0");
                        self.store(&mut local_params, 0)?;
                    }
                    ISTORE_1 | LSTORE_1 | DSTORE_1 | ASTORE_1 | FSTORE_1 => {
                        println!("STORE_1");
                        self.store(&mut local_params, 1)?;
                    }
                    ISTORE_2 | LSTORE_2 | DSTORE_2 | ASTORE_2 | FSTORE_2 => {
                        println!("STORE_2");
                        self.store(&mut local_params, 2)?;
                    }
                    ISTORE_3 | LSTORE_3 | DSTORE_3 | ASTORE_3 | FSTORE_3 => {
                        println!("STORE_3");
                        self.store(&mut local_params, 3)?;
                    }
                    BASTORE | IASTORE | LASTORE | CASTORE | SASTORE | FASTORE | DASTORE | AASTORE => unsafe {
                        println!("ASTORE");
                        self.array_store()?
                    },
                    POP => {
                        println!("POP");
                        self.current_frame().pop()?;
                    }
                    DUP => {
                        println!("DUP");
                        let value = self.current_frame().pop()?;
                        self.current_frame().push_ref(value.clone());
                        self.current_frame().push_ref(value);
                    }
                    IF_ICMPEQ | IF_ICMPNE | IF_ICMPGT | IF_ICMPGE | IF_ICMPLT | IF_ICMPLE => {

                        let jmp_to = read_u16(&code.opcodes, pc);
                        let value1 = self.current_frame().pop()?;
                        let value2 = self.current_frame().pop()?;
                        unsafe {
                            if let I32(value1) = *value1.get() {
                                if let I32(value2) = *value2.get() {
                                    let jump = match opcode {
                                        IF_ICMPEQ => { value1 == value2 }
                                        IF_ICMPNE => { value1 != value2 }
                                        IF_ICMPGT => { value1 > value2 }
                                        IF_ICMPGE => { value1 >= value2 }
                                        IF_ICMPLT => { value1 < value2 }
                                        IF_ICMPLE => { value1 <= value2 }
                                        _ => { false }
                                    };
                                    if jump {
                                        println!("JMP {}", jmp_to);
                                        *pc = jmp_to as usize;
                                    } else {
                                        println!("NO JMP {}", jmp_to);
                                    }
                                }
                            }
                        }
                    }
                    GOTO => {
                        let jmp_to = read_u16(&code.opcodes, pc);
                        println!("GOTO {}", jmp_to);
                        *pc = jmp_to as usize;
                    }
                    IRETURN | FRETURN | DRETURN => {
                        println!("RETURN");
                        let result = self.current_frame().pop();
                        self.stackframes.pop();
                        return result;
                    }
                    RETURN_VOID => {
                        println!("RETURN void");
                        self.stackframes.pop();
                        return Ok(Value::void());
                    }
                    GETSTATIC => {
                        println!("GETSTATIC");
                        let borrow = this_class.borrow();
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            borrow.cp_field_ref(&cp_index).unwrap(); // all these unwraps are safe as long as the class is valid
                        let (name_index, _) = borrow.cp_name_and_type(field_name_and_type_index).unwrap();
                        let (name) = borrow.cp_utf8(name_index).unwrap();

                        let that_class_name_index = borrow.cp_class_ref(class_index).unwrap();
                        let that_class_name = borrow.cp_utf8(that_class_name_index).unwrap();
                        let that = get_class(self, Some(&borrow.name), that_class_name.as_str())?;
                        let that_borrow = that.borrow();
                        let (_, val_index) = that_borrow.static_field_mapping.get(that_class_name).unwrap().get(name).unwrap();
                        println!("get static field {}", name);
                        self.current_frame().push_ref(that_borrow.static_data.get(*val_index).unwrap().as_ref().unwrap().clone());
                    }
                    PUTSTATIC => {
                        println!("PUTSTATIC");
                        let mut borrow = this_class.borrow_mut();
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            borrow.cp_field_ref(&cp_index).unwrap(); // all these unwraps are safe as long as the class is valid
                        let (name_index, _) = borrow.cp_name_and_type(field_name_and_type_index).unwrap();
                        let (name) = borrow.cp_utf8(name_index).unwrap();
                        let class_name_index = borrow.cp_class_ref(class_index).unwrap();
                        println!("field {}", name);
                        let that_class_name = borrow.cp_utf8(class_name_index).unwrap();

                        if &borrow.name == that_class_name {// may have to apply this in GETSTATIC too
                            let (_, val_index) = borrow.static_field_mapping.get(that_class_name).unwrap().get(name).as_ref().unwrap();
                            let val_index = *val_index;
                            let value = self.current_frame().pop()?;
                            borrow.static_data[val_index] = Some(value);
                        } else {
                            let that = get_class(self, Some(&borrow.name), that_class_name.as_str())?;
                            let that_borrow = that.borrow(); // if already borrowed, then that_class == this_class
                            let (_, val_index) = that_borrow.static_field_mapping.get(that_class_name).unwrap().get(name).unwrap();
                            let value = self.current_frame().pop()?;
                            borrow.static_data[*val_index] = Some(value);
                        }
                    }
                    GETFIELD => unsafe {
                        println!("PUTFIELD");
                        let borrow = this_class.borrow();
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            borrow.cp_field_ref(&cp_index).unwrap();
                        let (field_name_index, _) =
                            borrow.cp_name_and_type(field_name_and_type_index).unwrap();
                        let class_name_index = borrow.cp_class_ref(class_index).unwrap();
                        let class_name = borrow.cp_utf8(class_name_index).unwrap();
                        let field_name = borrow.cp_utf8(field_name_index).unwrap();

                        let mut objectref = self.current_frame().pop()?;
                        if let Value::Ref(instance) = &mut *objectref.get() {
                            if let ObjectRef::Object(ref mut object) = &mut *instance.get() {
                                let value = object.get(class_name, field_name);
                                self.current_frame().push_ref(Arc::clone(value));
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    },
                    PUTFIELD => unsafe {
                        println!("PUTFIELD");
                        let borrow = this_class.borrow();
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            borrow.cp_field_ref(&cp_index).unwrap();
                        let (field_name_index, _) =
                            borrow.cp_name_and_type(field_name_and_type_index).unwrap();
                        let class_name_index = borrow.cp_class_ref(class_index).unwrap();
                        let class_name = borrow.cp_utf8(class_name_index).unwrap();
                        let field_name = borrow.cp_utf8(field_name_index).unwrap();

                        let value = self.current_frame().pop()?;
                        let mut objectref = self.current_frame().pop()?;
                        if let Value::Ref(instance) = &mut *objectref.get() {
                            if let ObjectRef::Object(ref mut object) = &mut *instance.get() {
                                object.set(class_name, field_name, value);
                            }
                        } else {
                            unreachable!()
                        }
                    },
                    INVOKEVIRTUAL | INVOKESPECIAL => unsafe {
                        println!("INVOKE");
                        // TODO differentiate these opcodes
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) =
                            get_signature_for_invoke(&method.constant_pool, cp_index)
                        {
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, copy(self.current_frame().pop()?));
                            }
                            args.insert(0, self.current_frame().pop()?);
                            let return_value = self.execute(
                                Some(this_class.borrow().name.as_str()),
                                &invocation.class_name,
                                &invocation.method.name,
                                args,
                            )?;
                            match *return_value.get() {
                                Void => {}
                                _ => {
                                    self.current_frame().push_ref(return_value.clone());
                                }
                            }
                            println!("stack {} at {}", self.current_frame().len(), self.current_frame().at)
                        } else {
                            unreachable!()
                        }
                    },
                    INVOKESTATIC => unsafe {
                        println!("INVOKESTATIC");
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) =
                            get_signature_for_invoke(&method.constant_pool, cp_index)
                        {
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, copy(self.current_frame().pop()?));
                            }
                            let returnvalue = self.execute(
                                Some(this_class.borrow().name.as_str()),
                                &invocation.class_name,
                                &invocation.method.name,
                                args,
                            )?;
                            match *returnvalue.get() {
                                Void => {}
                                _ => {
                                    self.current_frame().push_ref(returnvalue.clone());
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    },
                    NEW => {
                        println!("NEW");
                        let class_index = &read_u16(&code.opcodes, pc);
                        let borrow = this_class.borrow();
                        let class_name_index = borrow.cp_class_ref(class_index).unwrap();
                        let class_name = borrow.cp_utf8(class_name_index).unwrap();
                        let class_to_instantiate = get_class(self, Some(&borrow.name), class_name)?;

                        let object = unsafe_ref(ObjectRef::Object(Box::new(
                            Vm::new_instance(class_to_instantiate),
                        )));
                        self.current_frame().push(Value::Ref(Arc::clone(&object)));
                        self.heap.new_object(object);
                    }
                    ANEWARRAY => unsafe {
                        println!("ANEWARRAY");
                        let class_index = &read_u16(&code.opcodes, pc);
                        let borrow = this_class.borrow();
                        let class_name_index = borrow.cp_class_ref(class_index).unwrap();
                        let class_name = borrow.cp_utf8(class_name_index).unwrap();
                        let arraytype = get_class(self, Some(&borrow.name), class_name)?;
                        let count = self.current_frame().pop()?;
                        if let I32(count) = *count.get() { // why does pop()?.get() give weird results?
                            let array = ObjectRef::new_object_array(arraytype, count as usize);
                            let array = unsafe_ref(array);

                            self.current_frame().push(Value::Ref(Arc::clone(&array)));
                            println!("{}", self.current_frame().len());
                            self.heap.new_object(array);
                        } else {
                            panic!();
                        }
                    }

                    //TODO implement all opcodes
                    _ => {
                        panic!("opcode {} not implemented {:?}", opcode, self.stackframes)
                        //TODO implement proper --stacktraces-- error handling
                    }
                }
            }
        }
        panic!("should not happen")
    }

    unsafe fn array_load(&mut self) -> Result<(), Error> {
        let value = self.current_frame().pop()?;

        if let I32(index) = &*value.get() {
            let index = *index as usize;
            let arrayref = self.current_frame().pop()?;
            if let Value::Null = &*arrayref.get() {
                return Err(anyhow!("NullpointerException"));
            }
            if let Value::Ref(objectref) = &*arrayref.get() {
                match &*objectref.get() {
                    ObjectRef::ByteArray(array) => {
                        self.current_frame().push(I32(array[index] as i32));
                    }
                    ObjectRef::ShortArray(array) => {
                        self.current_frame().push(I32(array[index] as i32));
                    }
                    ObjectRef::IntArray(array) => {
                        self.current_frame().push(I32(array[index]));
                    }
                    ObjectRef::BooleanArray(array) => {
                        self.current_frame().push(I32(array[index] as i32));
                    }
                    ObjectRef::CharArray(array) => {
                        self.current_frame().push(CHAR(array[index]));
                    }
                    ObjectRef::LongArray(array) => {
                        self.current_frame().push(Value::I64(array[index]));
                    }
                    ObjectRef::FloatArray(array) => {
                        self.current_frame().push(Value::F32(array[index]));
                    }
                    ObjectRef::DoubleArray(array) => {
                        self.current_frame().push(Value::F64(array[index]));
                    }
                    ObjectRef::ObjectArray(_arraytype, data) => {
                        self.current_frame()
                            .push(Value::Ref(data[index].clone()));
                    }
                    ObjectRef::Object(_) => { panic!("should be array") } //throw error?
                }
            }
        }
        Ok(())
    }

    unsafe fn array_store(&mut self) -> Result<(), Error> {
        let value = self.current_frame().pop()?;
        let index = &*self.current_frame().pop()?;
        let mut arrayref = &mut self.current_frame().pop()?;

        if let Value::Null = &*arrayref.get() {
            return Err(anyhow!("NullpointerException"));
        }

        if let I32(index) = &*index.get() {
            if let Value::Ref(ref mut objectref) = &mut *arrayref.get() {
                match &mut *objectref.get() {
                    ObjectRef::ByteArray(ref mut array) => {
                        if let I32(value) = *value.get() {
                            // is i32 correct?
                            array[*index as usize] = value as i8;
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::ShortArray(ref mut array) => {
                        if let I32(value) = *value.get() {
                            // is i32 correct?
                            array[*index as usize] = value as i16;
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::IntArray(ref mut array) => {
                        if let I32(value) = *value.get() {
                            array[*index as usize] = value;
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::BooleanArray(ref mut array) => {
                        if let I32(value) = *value.get() {
                            array[*index as usize] = value > 0;
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::CharArray(ref mut array) => {
                        if let I32(value) = *value.get() {
                            array[*index as usize] = char::from_u32_unchecked(value as u32);
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::LongArray(ref mut array) => {
                        if let Value::I64(value) = *value.get() {
                            array[*index as usize] = value;
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::FloatArray(ref mut array) => {
                        if let Value::F32(value) = *value.get() {
                            array[*index as usize] = value
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::DoubleArray(ref mut array) => {
                        if let Value::F64(value) = *value.get() {
                            array[*index as usize] = value
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::ObjectArray(arraytype, ref mut array) => {
                        if let Value::Ref(ref value) = *value.get() {
                            array[*index as usize] = value.clone();
                        } else {
                            unreachable!()
                        }
                    }
                    ObjectRef::Object(_) => {} //throw error?
                }
            }
        } else {
            unreachable!()
        }
        Ok(())
    }

    fn store(
        &mut self,
        local_params: &mut Vec<Option<UnsafeValue>>,
        index: usize,
    ) -> Result<(), Error> {
        let value = self.current_frame().pop()?;
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

// TODO can be simplified now, using cp_ methods in Class
fn get_signature_for_invoke(cp: &Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<Invocation> {
    if let CpEntry::MethodRef(class_index, name_and_type_index)
    | CpEntry::InterfaceMethodref(class_index, name_and_type_index) = cp.get(&index).unwrap()
    {
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

unsafe fn copy(value: UnsafeValue) -> UnsafeValue {
    unsafe_val(match (&*value.get()) {
        Void => Void,
        Null => Null,
        BOOL(b) => BOOL(*b),
        CHAR(c) => CHAR(*c),
        I32(i) => I32(*i),
        I64(l) => I64(*l),
        F32(f) => F32(*f),
        F64(d) => F64(*d),
        Ref(r) => Ref(r.clone()),
        Utf8(s) => Utf8(s.to_owned()),
    })
}

fn get_name_and_type(cp: Rc<HashMap<u16, CpEntry>>, index: u16) -> Option<MethodSignature> {
    if let CpEntry::NameAndType(method_name_index, signature_index) = cp.get(&index).unwrap() {
        if let CpEntry::Utf8(method_name) = cp.get(method_name_index).unwrap() {
            if let CpEntry::Utf8(signature) = cp.get(signature_index).unwrap() {
                let mut method_signature: String = method_name.into();
                let num_args = get_hum_args(signature);
                method_signature.push_str(signature);
                return Some(MethodSignature {
                    name: method_signature,
                    num_args,
                });
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
            i += 1;
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
