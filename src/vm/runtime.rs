use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Error;
use log::debug;

use crate::class::ClassId;
use crate::classloader::classdef::{CpEntry, Modifier};
use crate::classloader::io::PATH_SEPARATOR;
use crate::classmanager::ClassManager;
use crate::value::Value::{self, *};
use crate::vm::array::array_load;
use crate::vm::object;
use crate::vm::object::ObjectRef;
use crate::vm::object::ObjectRef::Object;
use crate::vm::opcodes::Opcode;
use crate::vm::opcodes::Opcode::*;
use std::io::Write;
use crate::vm::native::invoke_native;

pub struct Vm {
    pub stack: Vec<Stackframe>,
}

impl Vm {
    pub fn new() -> Self {
        env_logger::builder()
            .format(|buf, record| {
                writeln!(buf, "{}: {}", record.level(), record.args())
            })
            .try_init().unwrap();
        Self {
            stack: vec![]
        }
    }

    pub fn run(mut self, classpath: &str, class_name: &str, method_name: &str) {
        let classpath = classpath.split(PATH_SEPARATOR).map(|s| s.into())
            .collect();
        let mut class_manager = ClassManager::new(classpath);

        class_manager.load_class_by_name("java/lang/Class");
        class_manager.load_class_by_name("java/lang/System");
        class_manager.load_class_by_name("java/lang/String");
        class_manager.load_class_by_name("java/util/Collections");


        class_manager.load_class_by_name(class_name);
        let system_id = *class_manager.get_classid("java/lang/System");
        self.run2(&mut class_manager, system_id, "initPhase1()V");
        // let class_id = *class_manager.get_classid(class_name);
        // self.run2(&mut class_manager, class_id, method_name);
    }

    pub(crate) fn run2(&mut self, class_manager: &mut ClassManager, class_id: ClassId, method_name: &str) {
        Stackframe::default().run(class_manager, class_id, method_name);
    }
}

pub struct Stackframe {
    pc: usize,
    locals: Vec<Value>,
    stack: Vec<Value>,
}

impl Stackframe {
    pub fn new(args: Vec<Value>) -> Self {
        Self {
            pc: 0,
            locals: args,
            stack: vec![],
        }
    }

    pub fn default() -> Self {
        Self {
            pc: 0,
            locals: vec![],
            stack: vec![],
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn run(&mut self, class_manager: &mut ClassManager, class_id: ClassId, method_name: &str) -> Value {
        let classname = class_manager.get_class_by_id(&class_id).unwrap().name.to_owned();

        let code = class_manager.get_classdef(&class_id).get_method(method_name).unwrap().code.clone();
        let constant_pool = class_manager.get_classdef(&class_id).get_method(method_name).unwrap().constant_pool.clone();

        let len = code.len();
        while self.pc < len {
            let opcode: &Opcode = code.get(self.pc).unwrap();
            debug!("\tat {}.{}: {} #{:?} - {:?}", classname, method_name, self.pc, opcode, self.stack);
            match opcode {
                NOP => {}
                ACONST_NULL => {
                    self.push(Null);
                }
                ICONST(v) => {
                    self.push(I32(*v as i32));
                }
                LCONST(v) => {
                    self.push(I64(*v as i64));
                }
                FCONST(v) => {
                    self.push(F32(*v as f32));
                }
                DCONST(v) => {
                    self.push(F64(*v as f64));
                }
                SIPUSH(si) => {
                    self.push(I32(*si as i32));
                }
                BIPUSH(bi) => {
                    self.push(I32(*bi as i32));
                }
                LDC(c) | LDC_W(c) | LDC2_W(c) => {
                    let c = constant_pool.get(&c).unwrap();
                    match c {
                        CpEntry::Integer(i) => {
                            self.push(I32(*i));
                        }
                        CpEntry::Float(f) => {
                            self.push(F32(*f));
                        }
                        CpEntry::Double(d) => {
                            self.push(F64(*d));
                        }
                        CpEntry::StringRef(utf8) => {
                            //TODO
                            let string = class_manager.get_classdef(&class_id).cp_utf8(&utf8);
                            let string: Vec<u8> = string.as_bytes().into();
                            class_manager.load_class_by_name("java/lang/String");
                            let stringclass = class_manager.get_class_by_name("java/lang/String").unwrap();
                            let mut stringinstance = object::Object::new(stringclass);
                            stringinstance.set(stringclass, "java/lang/String", "value", Ref(ObjectRef::new_byte_array(string)));

                            self.push(Ref(Object(Rc::new(RefCell::new(stringinstance)))));
                        }
                        CpEntry::Long(l) => {
                            self.push(I64(*l));
                        }
                        CpEntry::ClassRef(utf8_index) => {
                            let class_name = class_manager.get_classdef(&class_id).cp_utf8(&utf8_index).to_owned();
                            class_manager.load_class_by_name(&class_name);
                            let klass_id = class_manager.get_classid(&class_name);
                            if let Some(class) = class_manager.get_classobject(klass_id) {
                                self.push(class.clone());
                            } else {
                                unreachable!("should not be here");
                            }
                        }
                        _ => {
                            panic!("add variant {:?}", c)
                        }
                    }
                }
                ILOAD(n) | LLOAD(n) | FLOAD(n) | DLOAD(n) | ALOAD(n) => {
                    // omitting the type checks so far
                    self.push(self.locals[*n as usize].clone());
                }
                IALOAD | LALOAD | FALOAD | DALOAD | AALOAD | BALOAD | CALOAD | SALOAD => {
                    let index = self.pop();
                    let arrayref = self.pop();
                    self.push(array_load(index, arrayref).unwrap()); //TODO errorhandling
                }
                ISTORE(c) | LSTORE(c) | FSTORE(c) | DSTORE(c) | ASTORE(c) => {
                    self.store(*c).unwrap();
                }
                INVOKEVIRTUAL(c) => {
                    if let Some(invocation) =
                        get_signature_for_invoke(&constant_pool, *c)
                    {
                        let mut args = Vec::with_capacity(invocation.method.num_args);
                        for _ in 0..invocation.method.num_args {
                            args.insert(0, self.pop().clone());
                        }
                        let this_ref = self.pop();
                        args.insert(0, this_ref.clone());


                        debug!("invoke {:?}", invocation);
                        let mut invoke_class: Option<ClassId> = None;
                        if let Null = this_ref {
                            panic!("NullPointer Exception");
                        }
                        if let Ref(this) = this_ref {
                            if let Object(this) = this {
                                let invoke_classdef = class_manager.get_classdef(&this.borrow().class_id);
                                let invoke_method = invoke_classdef.get_method(&invocation.method.name);
                                if invoke_method.is_some() {
                                    class_manager.load_class_by_name(&invocation.class_name);
                                    invoke_class = Some(*class_manager.get_classid(&invocation.class_name));
                                } else {
                                    let name = class_manager.classdef_name(&this.borrow().class_id);
                                    if let Some(name) = name {
                                        class_manager.load_class_by_name(&name);
                                        let class = class_manager.get_class_by_name(&name).unwrap();

                                        for parent_id in &class.parents {
                                            let classdef = class_manager.get_classdef(parent_id);
                                            if classdef.has_method(method_name) {
                                                invoke_class = Some(*parent_id);
                                                break;
                                            }
                                        }
                                    } else {
                                        panic!("ClassNotFound");
                                    }
                                }
                            } else if let ObjectRef::Class(_class) = this { // special case for Class ?
                                invoke_class = Some(*class_manager.get_classid("java/lang/Class"));
                            }
                        }
                        if invoke_class.is_none() {
                            panic!("method {:?}.{} not found", invocation.class_name, invocation.method.name);
                        }

                        let return_value =
                            if class_manager.get_classdef(&invoke_class.unwrap()).get_method(&invocation.method.name).unwrap().is(Modifier::Native) {
                                invoke_native(class_manager, invocation.class_name.as_str(), invocation.method.name.as_str(), args).unwrap()
                                // TODO remove unwrap in line above, error handling
                            } else {
                                let mut new_stackframe = Stackframe::new(args);
                                new_stackframe.run(class_manager, invoke_class.unwrap(), &invocation.method.name)
                            };
                        match return_value {
                            Void => {}
                            _ => self.push(return_value)
                        }
                    } else {
                        unreachable!()
                    }
                }
                INVOKESPECIAL(c) | INVOKESTATIC(c) => {
                    if let Some(invocation) =
                        get_signature_for_invoke(&constant_pool, *c)
                    {
                        debug!("invoke {:?}", invocation);
                        let mut args = Vec::with_capacity(invocation.method.num_args);
                        for _ in 0..invocation.method.num_args {
                            args.insert(0, self.pop().clone());
                        }
                        if let INVOKESPECIAL(_) = opcode {
                            args.insert(0, self.pop());
                        }

                        class_manager.load_class_by_name(invocation.class_name.as_str());
                        let invoke_class = class_manager.get_classid(invocation.class_name.as_str());

                        let return_value =
                            if class_manager.get_classdef(&invoke_class).get_method(&invocation.method.name).unwrap().is(Modifier::Native) {
                                invoke_native(class_manager, invocation.class_name.as_str(), invocation.method.name.as_str(), args).unwrap()
                                // TODO remove unwrap in line above, error handling
                            } else {
                                let mut new_stackframe = Stackframe::new(args);
                                new_stackframe.run(class_manager, *invoke_class, &invocation.method.name)
                            };
                        debug!("returning {:?}", return_value);
                        match return_value {
                            Void => {}
                            _ => self.push(return_value)
                        }
                    } else {
                        unreachable!()
                    }
                }
                GETSTATIC(field_index) => {
                    let classdef = class_manager.get_classdef(&class_id);
                    let (class_index, field_name_and_type_index) =
                        classdef.cp_field_ref(&field_index); // all these unwraps are safe as long as the class is valid
                    let (name_index, _) =
                        classdef.cp_name_and_type(field_name_and_type_index);
                    let field_name = classdef.cp_utf8(name_index).to_owned();
                    let that_class_name = classdef.cp_utf8(classdef.cp_class_ref(class_index)).to_owned();
                    class_manager.load_class_by_name(&that_class_name);
                    let that_class = class_manager.get_class_by_name(&that_class_name).unwrap();

                    let type_index = that_class
                        .static_field_mapping
                        .get(&that_class_name)
                        .unwrap()// safe because class for static field must be there
                        .get(&field_name)
                        .unwrap(); // safe because field must be there

                    debug!("get_static {}.{}", that_class_name, field_name);
                    let field_value = class_manager.get_static(&that_class.id, type_index.index);
                    self.push(field_value);
                }
                PUTSTATIC(field_index) => {
                    let classdef = class_manager.get_classdef(&class_id);
                    let (class_index, field_name_and_type_index) =
                        classdef.cp_field_ref(&field_index); // all these unwraps are safe as long as the class is valid
                    let (name_index, _) =
                        classdef.cp_name_and_type(field_name_and_type_index);
                    let name = classdef.cp_utf8(name_index);
                    let that_class_name_index = classdef.cp_class_ref(class_index);
                    let that_class_name = classdef.cp_utf8(that_class_name_index);
                    let that_class = class_manager.get_class_by_name(that_class_name).unwrap();
                    let val_index = that_class.static_field_mapping
                        .get(that_class_name)
                        .unwrap()
                        .get(name)
                        .unwrap()
                        .index;
                    class_manager.set_static(class_id, val_index, self.pop());
                }
                GETFIELD(field_index) => {
                    let classdef = class_manager.get_classdef(&class_id);
                    let (class_index, field_name_and_type_index) =
                        classdef.cp_field_ref(&field_index);
                    let (field_name_index, _) =
                        classdef.cp_name_and_type(field_name_and_type_index);
                    let declared_type = classdef.cp_utf8(classdef.cp_class_ref(class_index)).to_owned();

                    let field_name = classdef.cp_utf8(field_name_index).to_owned();
                    debug!("get field {}.{}",declared_type, field_name);
                    let objectref = self.pop();
                    if let Ref(instance) = objectref {
                        if let Object(object) = instance {
                            let runtime_type = class_manager.get_class_by_id(&object.borrow().class_id).unwrap();
                            let object = object.borrow();
                            let value = object.get(runtime_type, &declared_type, &field_name);
                            self.push(value.clone());
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!("objectref {:?}", objectref)
                    }
                }
                PUTFIELD(field_index) => {
                    let classdef = class_manager.get_classdef(&class_id);
                    let (class_index, field_name_and_type_index) =
                        classdef.cp_field_ref(&field_index);
                    let (field_name_index, _) =
                        classdef.cp_name_and_type(field_name_and_type_index);
                    let class_name_index = classdef.cp_class_ref(class_index);
                    let declared_type = classdef.cp_utf8(class_name_index).to_owned();
                    let field_name = classdef.cp_utf8(field_name_index).to_owned();

                    let value = self.pop();
                    let objectref = self.pop();
                    if let Ref(instance) = objectref {
                        if let Object(object) = instance {
                            let runtime_type = class_manager.get_class_by_id(&object.borrow().class_id).unwrap();
                            object.borrow_mut().set(runtime_type, &declared_type, &field_name, value);
                        }
                    } else {
                        unreachable!()
                    }
                }
                NEW(class_index) => {
                    let class_name_index = *class_manager.get_classdef(&class_id).cp_class_ref(class_index);
                    let class_name = class_manager.get_classdef(&class_id).cp_utf8(&class_name_index).to_owned();
                    class_manager.load_class_by_name(&class_name);
                    let class_to_instantiate = class_manager.get_class_by_name(&class_name).unwrap();

                    let object = Object(Rc::new(RefCell::new(object::Object::new(
                        class_to_instantiate,
                    ))));
                    self.push(Ref(object));
                }
                NEWARRAY(arraytype) => {
                    let count = self.pop();
                    let array = ObjectRef::new_array(*arraytype, count.into_i32() as usize);
                    self.push(Ref(array));
                }
                ANEWARRAY(class_index) => {
                    let class_name_index = *class_manager.get_classdef(&class_id).cp_class_ref(class_index);
                    let class_name = class_manager.get_classdef(&class_id).cp_utf8(&class_name_index).to_owned();
                    class_manager.load_class_by_name(&class_name);
                    let arraytype = class_manager.get_class_by_name(&class_name).unwrap();
                    let count = self.pop().into_i32();
                    let array = ObjectRef::new_object_array(arraytype, count as usize);
                    self.push(Ref(array));
                }
                ARRAYLENGTH => {
                    let val = self.pop();
                    if let Ref(val) = val {
                        self.push(I32(val.get_array_length() as i32));
                    } else {
                        unreachable!("array length {:?}", val);
                    }
                }
                MONITORENTER | MONITOREXIT => {
                    self.pop();
                } //TODO implement
                IFNULL(_) | IFNONNULL(_) => {
                    let value = self.pop();
                    match value {
                        Null => if let IFNULL(goto) = opcode { self.pc = *goto as usize; }
                        _ => if let IFNONNULL(goto) = opcode { self.pc = *goto as usize; }
                    };
                }
                DUP => {
                    let value = self.pop();
                    self.push(value.clone());
                    self.push(value.clone());
                }
                IRETURN | LRETURN | FRETURN | DRETURN | ARETURN => {
                    return self.pop();
                }
                RETURN_VOID => {
                    return Void;
                }
                _ => { panic!("opcode not implemented") }
            }
            self.pc += 1;
        }
        Void
    }

    fn store(
        &mut self,
        index: u8,
    ) -> Result<(), Error> {
        let index = index as usize;
        let value = self.pop();
        while self.locals.len() < index + 1 {
            self.locals.push(Null); //ensure capacity
        }
        self.locals[index] = value;
        Ok(())
    }
}

pub(crate) fn get_signature_for_invoke(cp: &HashMap<u16, CpEntry>, index: u16) -> Option<Invocation> {
    if let CpEntry::MethodRef(class_index, name_and_type_index)
    | CpEntry::InterfaceMethodref(class_index, name_and_type_index) = cp.get(&index).unwrap()
    {
        if let Some(method_signature) = get_name_and_type(cp, *name_and_type_index) {
            if let CpEntry::ClassRef(class_name_index) = cp.get(class_index).unwrap() {
                if let CpEntry::Utf8(class_name) = cp.get(class_name_index).unwrap() {
                    return Some(Invocation::new(
                        class_name.into(),
                        method_signature)
                    );
                }
            }
        }
    }
    None
}

pub(crate) fn get_name_and_type(cp: &HashMap<u16, CpEntry>, index: u16) -> Option<MethodSignature> {
    if let CpEntry::NameAndType(method_name_index, signature_index) = cp.get(&index).unwrap() {
        if let CpEntry::Utf8(method_name) = cp.get(method_name_index).unwrap() {
            if let CpEntry::Utf8(signature) = cp.get(signature_index).unwrap() {
                let mut method_signature: String = method_name.into();
                let num_args = get_num_args(signature);
                method_signature.push_str(signature);
                return Some(MethodSignature::new(method_signature, num_args));
            }
        }
    }
    None
}

fn get_num_args(signature: &str) -> usize {
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
        } else if chars[i] == '[' {
            i += 1;
        } else {
            i += 1;
            num += 1;
        }
    }
    num
}

#[derive(Debug)]
pub(crate) struct Invocation {
    class_name: String,
    method: MethodSignature,
}

impl Invocation {
    pub fn new(class_name: String, method: MethodSignature) -> Self {
        Self {
            class_name,
            method,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MethodSignature {
    name: String,
    num_args: usize,
}

impl MethodSignature {
    pub(crate) fn new(name: String, num_args: usize) -> Self {
        MethodSignature {
            name,
            num_args,
        }
    }
}