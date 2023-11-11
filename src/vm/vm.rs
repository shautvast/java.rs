use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use anyhow::Error;
use log::{debug, error};

use crate::class::{Class, Object, ObjectRef, Value};
use crate::class::Value::{F32, F64, I32, I64, Null, Ref, Utf8, Void};
use crate::classloader::classdef::{AttributeType, CpEntry, Method, Modifier};
use crate::classloader::io::{read_u16, read_u8};
use crate::classmanager;
use crate::classmanager::get_class_by_id;
use crate::vm::array::{array_load, array_store};
use crate::vm::native::invoke_native;
use crate::vm::opcodes;
use crate::vm::opcodes::*;
use crate::vm::operations::{get_signature_for_invoke, get_static, load_constant};
use crate::vm::stack::StackFrame;

pub struct Vm {}


#[cfg(target_family = "unix")]
const PATH_SEPARATOR: char = ':';

#[cfg(target_family = "windows")]
const PATH_SEPARATOR: char = ';';

const MASK_LOWER_5BITS: i32 = 0b00011111;

/// The singlethreaded VM (maybe a future Thread)
//TODO goto
//TODO error handling
impl Vm {
    /// for running static initializers
    pub fn new_internal() -> Self {
        Self {}
    }

    pub fn new(stack: &mut Vec<StackFrame>) -> Self {
        env_logger::builder()
            .format(|buf, record| {
                writeln!(buf, "{}: {}", record.level(), record.args())
            })
            .try_init().unwrap();
        let mut vm_instance = Self {};
        classmanager::init();
        Vm::init(&mut vm_instance, stack);
        vm_instance
    }

    fn init(vm: &mut Vm, stack: &mut Vec<StackFrame>) {
        classmanager::load_class_by_name("java/lang/Class");
        vm.execute_static(stack, "java/lang/System", "initPhase1()V", vec![]).expect("cannot create VM");
    }

    pub fn new_instance(class: &Class) -> Object {
        Object::new(class)
    }

    /// execute the bytecode
    pub fn execute_virtual(
        &mut self,
        stack: &mut Vec<StackFrame>,
        class_name: &str,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        for arg in &args {
            if let Ref(r) = arg {
                debug!("arg {:?}",r);
            } else {
                debug!("arg {:?}",arg);
            }
        }

        if let Null = args[0] {
            panic!("NPE");
        }
        if let Ref(this) = &args[0] {
            if let ObjectRef::Object(this) = this {
                let thisb= this.borrow();
                let cd = classmanager::get_classdef(&thisb.class_id);
                let method = cd.get_method(method_name);
                if let Some(method) = method {
                    classmanager::load_class_by_name(class_name);
                    let class = classmanager::get_class_by_name(class_name).unwrap();
                    return self.execute_class(stack, class, method.name().as_str(), args.clone());
                } else {
                    let name = classmanager::classdef_name(&this.borrow().class_id);
                    if let Some(name) = name {
                        classmanager::load_class_by_name(&name);
                        let class = classmanager::get_class_by_name(&name).unwrap();

                        for parent_id in &class.parents {
                            let classdef = classmanager::get_classdef(parent_id);
                            let method = classdef.get_method(method_name);
                            if let Some(_) = method {
                                let class= get_class_by_id(parent_id).unwrap();
                                return self.execute_class(stack, class, method_name, args.clone());
                            }
                        }
                    } else {
                        panic!("ClassNotFound");
                    }
                }
            } else if let ObjectRef::Class(_class) = this {
                //TODO is this right??
                classmanager::load_class_by_name("java/lang/Class");//TODO preload, so this is not needed
                let klazz = classmanager::get_class_by_name("java/lang/Class").unwrap();
                // let klazzdef = self.classmanager.get_classdef(&klazz.id);
                return self.execute_class(stack, klazz, method_name, args);
            }
        }
        println!("this is not an object reference {}", class_name);
        panic!();
    }

    pub fn execute_special(
        &mut self,
        stack: &mut Vec<StackFrame>,
        class_name: &str,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        classmanager::load_class_by_name(class_name);
        let class = classmanager::get_class_by_name(class_name).unwrap();
        self.execute_class(stack, class, method_name, args)
    }

    pub fn execute_static(
        &mut self,
        stack: &mut Vec<StackFrame>,
        class_name: &str,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        classmanager::load_class_by_name(class_name);
        let class = classmanager::get_class_by_name(class_name).unwrap();
        self.execute_class(stack, class, method_name, args)
    }

    pub fn execute_class(
        &mut self,
        stackframes: &mut Vec<StackFrame>,
        this_class: &Class,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        debug!("execute {}.{}", this_class.name, method_name);

        //TODO implement dynamic dispatch -> get method from instance
        let method = classmanager::get_classdef(&this_class.id).get_method(method_name).unwrap();
        let mut local_params: Vec<Option<Value>> =
            args.clone().iter().map(|e| Some(e.clone())).collect();
        if method.is(Modifier::Native) {
            return invoke_native(self, stackframes, &this_class.name, method_name, args);
        }
        if let AttributeType::Code(code) = method.attributes.get("Code").unwrap() {
            let stackframe = StackFrame::new(&this_class.name, method_name);
            stackframes.push(stackframe);

            let pc = &mut 0;
            while *pc < code.opcodes.len() {
                let opcode = read_u8(&code.opcodes, pc);
                let cur_frame = current_frame(stackframes);
                debug!("\t{} #{} {} - {:?}", &cur_frame.at, &*pc - 1, opcodes::OPCODES[opcode as usize], cur_frame.data);
                match opcode {
                    ACONST_NULL => {
                        current_frame(stackframes).push(Value::Null);
                    }
                    ICONST_M1 => {
                        current_frame(stackframes).push(I32(-1));
                    }
                    ICONST_0 => {
                        current_frame(stackframes).push(I32(0));
                    }
                    ICONST_1 => {
                        current_frame(stackframes).push(I32(1));
                    }
                    ICONST_2 => {
                        current_frame(stackframes).push(I32(2));
                    }
                    ICONST_3 => {
                        current_frame(stackframes).push(I32(3));
                    }
                    ICONST_4 => {
                        current_frame(stackframes).push(I32(4));
                    }
                    ICONST_5 => {
                        current_frame(stackframes).push(I32(5));
                    }
                    LCONST_0 => {
                        current_frame(stackframes).push(I64(0));
                    }
                    LCONST_1 => {
                        current_frame(stackframes).push(I64(1));
                    }
                    FCONST_0 => {
                        current_frame(stackframes).push(F32(0.0));
                    }
                    FCONST_1 => {
                        current_frame(stackframes).push(F32(1.0));
                    }
                    FCONST_2 => {
                        current_frame(stackframes).push(F32(2.0));
                    }
                    DCONST_0 => {
                        current_frame(stackframes).push(F64(0.0));
                    }
                    DCONST_1 => {
                        current_frame(stackframes).push(F64(1.0));
                    }
                    SIPUSH => {
                        let s = read_u16(&code.opcodes, pc) as i32;
                        current_frame(stackframes).push(I32(s));
                    }
                    BIPUSH => {
                        let c = read_u8(&code.opcodes, pc) as i32;
                        current_frame(stackframes).push(I32(c));
                    }
                    LDC => {
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        load_constant(&cp_index, method, stackframes, this_class);
                    }
                    LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        load_constant(&cp_index, method, stackframes, this_class);
                    }
                    LDC2_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        load_constant(&cp_index, method, stackframes, this_class);
                    }
                    ILOAD | LLOAD | FLOAD | DLOAD | ALOAD => {
                        // omitting the type checks so far
                        let n = read_u8(&code.opcodes, pc) as usize;
                        current_frame(stackframes)
                            .push(local_params[n].as_ref().unwrap().clone());
                    }
                    ILOAD_0 | LLOAD_0 | FLOAD_0 | DLOAD_0 | ALOAD_0 => {
                        current_frame(stackframes)
                            .push(local_params[0].as_ref().unwrap().clone());
                    }
                    ILOAD_1 | LLOAD_1 | FLOAD_1 | DLOAD_1 | ALOAD_1 => {
                        current_frame(stackframes)
                            .push(local_params[1].as_ref().unwrap().clone());
                    }
                    ILOAD_2 | LLOAD_2 | FLOAD_2 | DLOAD_2 | ALOAD_2 => {
                        current_frame(stackframes)
                            .push(local_params[2].as_ref().unwrap().clone());
                    }
                    ILOAD_3 | LLOAD_3 | FLOAD_3 | DLOAD_3 | ALOAD_3 => {
                        current_frame(stackframes)
                            .push(local_params[3].as_ref().unwrap().clone());
                    }
                    IALOAD | LALOAD | FALOAD | DALOAD | AALOAD | BALOAD | CALOAD | SALOAD => {
                        let index = current_frame(stackframes).pop()?;
                        let arrayref = current_frame(stackframes).pop()?;
                        current_frame(stackframes).push(array_load(index, arrayref)?);
                    }
                    ISTORE | LSTORE | FSTORE | DSTORE | ASTORE => {
                        let index = read_u8(&code.opcodes, pc) as usize;
                        self.store(stackframes, &mut local_params, index)?;
                    }
                    ISTORE_0 | LSTORE_0 | DSTORE_0 | ASTORE_0 | FSTORE_0 => {
                        self.store(stackframes, &mut local_params, 0)?;
                    }
                    ISTORE_1 | LSTORE_1 | DSTORE_1 | ASTORE_1 | FSTORE_1 => {
                        self.store(stackframes, &mut local_params, 1)?;
                    }
                    ISTORE_2 | LSTORE_2 | DSTORE_2 | ASTORE_2 | FSTORE_2 => {
                        self.store(stackframes, &mut local_params, 2)?;
                    }
                    ISTORE_3 | LSTORE_3 | DSTORE_3 | ASTORE_3 | FSTORE_3 => {
                        self.store(stackframes, &mut local_params, 3)?;
                    }
                    BASTORE | IASTORE | LASTORE | CASTORE | SASTORE | FASTORE | DASTORE
                    | AASTORE => {
                        let value = current_frame(stackframes).pop()?;
                        let index = current_frame(stackframes).pop()?;
                        let arrayref = current_frame(stackframes).pop()?;
                        array_store(value, index, arrayref)?
                    }
                    POP => {
                        current_frame(stackframes).pop()?;
                    }
                    DUP => {
                        let value = current_frame(stackframes).pop()?;
                        current_frame(stackframes).push(value.clone());
                        current_frame(stackframes).push(value);
                    }
                    IADD => {
                        let value2 = current_frame(stackframes).pop()?;
                        let value1 = current_frame(stackframes).pop()?;
                        debug!("{:?}+{:?}", value1, value2);
                        current_frame(stackframes).push(I32(value1.into_i32() + value2.into_i32()));
                    }
                    ISUB => {
                        let value2 = current_frame(stackframes).pop()?;
                        let value1 = current_frame(stackframes).pop()?;
                        debug!("{:?}-{:?}", value1, value2);
                        current_frame(stackframes).push(I32(value1.into_i32() - value2.into_i32()));
                    }
                    IDIV => {
                        let value2 = current_frame(stackframes).pop()?;
                        let value1 = current_frame(stackframes).pop()?;
                        current_frame(stackframes).push(I32(value1.into_i32() / value2.into_i32()));
                    }
                    ISHL => {
                        let value2 = current_frame(stackframes).pop()?;
                        let value1 = current_frame(stackframes).pop()?;
                        debug!("{:?} shl {:?}", value1, value2);
                        current_frame(stackframes).push(I32(value1.into_i32() << (value2.into_i32() & MASK_LOWER_5BITS)));
                    }
                    ISHR => {
                        let value2 = current_frame(stackframes).pop()?;
                        let value1 = current_frame(stackframes).pop()?;
                        debug!("{:?} shr {:?}", value1, value2);
                        current_frame(stackframes).push(I32(value1.into_i32() >> (value2.into_i32() & MASK_LOWER_5BITS)));
                    }
                    IFEQ | IFNE | IFLT | IFGE | IFGT | IFLE => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3; // -3 so that offset = location of Cmp opcode
                        let value = current_frame(stackframes).pop()?;
                        Self::if_cmp(pc, opcode, jmp_to, &value, &I32(0));
                    }

                    IF_ICMPEQ | IF_ICMPNE | IF_ICMPGT | IF_ICMPGE | IF_ICMPLT | IF_ICMPLE => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3; // -3 so that offset = location of Cmp opcode
                        let value1 = current_frame(stackframes).pop()?;
                        let value2 = current_frame(stackframes).pop()?;
                        Self::if_cmp(pc, opcode, jmp_to, &value1, &value2);
                    }
                    GOTO => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3;
                        *pc += jmp_to as usize;
                        debug!("GOTO {}", *pc)
                    }
                    IRETURN | FRETURN | DRETURN | ARETURN => {
                        let result = current_frame(stackframes).pop();
                        stackframes.pop();
                        return result;
                    }
                    RETURN_VOID => {
                        stackframes.pop();
                        return Ok(Void);
                    }
                    GETSTATIC => {
                        let field_index = read_u16(&code.opcodes, pc);
                        let field_value = get_static(this_class, field_index)?;

                        current_frame(stackframes).push(field_value);
                    }
                    PUTSTATIC => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            classdef.cp_field_ref(&cp_index); // all these unwraps are safe as long as the class is valid
                        let (name_index, _) =
                            classdef.cp_name_and_type(field_name_and_type_index);
                        let name = classdef.cp_utf8(name_index);
                        let that_class_name_index = classdef.cp_class_ref(class_index);
                        let that_class_name = classdef.cp_utf8(that_class_name_index);
                        let that_class = classmanager::get_class_by_name(that_class_name).unwrap();
                        let val_index = that_class.static_field_mapping
                            .get(that_class_name)
                            .unwrap()
                            .get(name)
                            .unwrap()
                            .index;
                        let value = current_frame(stackframes).pop()?;
                        classmanager::set_static(&this_class.id, val_index, value);
                    }
                    GETFIELD => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            classdef.cp_field_ref(&cp_index);
                        let (field_name_index, _) =
                            classdef.cp_name_and_type(field_name_and_type_index);
                        let class_name_index = classdef.cp_class_ref(class_index);
                        let declared_type = classdef.cp_utf8(class_name_index);

                        let field_name = classdef.cp_utf8(field_name_index);
                        debug!("get field {}.{}",declared_type, field_name);
                        let objectref = current_frame(stackframes).pop()?;
                        if let Ref(instance) = objectref {
                            if let ObjectRef::Object(object) = instance {
                                let runtime_type = classmanager::get_class_by_id(&object.borrow().class_id).unwrap();
                                let object = object.borrow();
                                let value = object.get(runtime_type, declared_type, field_name);
                                current_frame(stackframes).push(value.clone());
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!("objectref {:?}", objectref)
                        }
                    }
                    PUTFIELD => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            classdef.cp_field_ref(&cp_index);
                        let (field_name_index, _) =
                            classdef.cp_name_and_type(field_name_and_type_index);
                        let class_name_index = classdef.cp_class_ref(class_index);
                        let declared_type = classdef.cp_utf8(class_name_index);
                        let field_name = classdef.cp_utf8(field_name_index);

                        let value = current_frame(stackframes).pop()?;
                        let objectref = current_frame(stackframes).pop()?;
                        if let Ref(instance) = objectref {
                            if let ObjectRef::Object(object) = instance {
                                let runtime_type = classmanager::get_class_by_id(&object.borrow().class_id).unwrap();
                                object.borrow_mut().set(runtime_type, declared_type, field_name, value);
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    INVOKESPECIAL => {
                        // TODO differentiate these opcodes
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) =
                            get_signature_for_invoke(&method.constant_pool, cp_index)
                        {
                            debug!("invoke {:?}", invocation);
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, current_frame(stackframes).pop()?.clone());
                            }
                            args.insert(0, current_frame(stackframes).pop()?);
                            let return_value = self.execute_special(stackframes,
                                                                    &invocation.class_name,
                                                                    &invocation.method.name,
                                                                    args,
                            )?;
                            if let Ref(objectref) = &return_value {
                                if let ObjectRef::Object(object) = objectref {
                                    debug!("return {:?}", object);
                                }
                            } else {
                                debug!("return {:?}", return_value);
                            }
                            match return_value {
                                Void => {}
                                _ => {
                                    current_frame(stackframes).push(return_value.clone());
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    INVOKEVIRTUAL => {
                        // TODO differentiate these opcodes
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) =
                            get_signature_for_invoke(&method.constant_pool, cp_index)
                        {
                            debug!("invoke {:?}", invocation);
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, current_frame(stackframes).pop()?.clone());
                            }
                            args.insert(0, current_frame(stackframes).pop()?);
                            let return_value = self.execute_virtual(
                                stackframes,
                                &invocation.class_name,
                                &invocation.method.name,
                                args,
                            )?;
                            if let Ref(objectref) = &return_value {
                                if let ObjectRef::Object(object) = objectref {
                                    debug!("return {:?}", object);
                                }
                            } else {
                                debug!("return {:?}", return_value);
                            }
                            match return_value {
                                Void => {}
                                _ => {
                                    current_frame(stackframes).push(return_value.clone());
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    INVOKESTATIC => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        if let Some(invocation) =
                            get_signature_for_invoke(&method.constant_pool, cp_index)
                        {
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, current_frame(stackframes).pop()?.clone());
                            }
                            let return_value = self.execute_static(stackframes,
                                                                   &invocation.class_name,
                                                                   &invocation.method.name,
                                                                   args,
                            )?;
                            if let Ref(objectref) = &return_value {
                                if let ObjectRef::Object(object) = objectref {
                                    debug!("return {:?}", object);
                                }
                            } else {
                                debug!("return {:?}", return_value);
                            }
                            match return_value {
                                Void => {}
                                _ => {
                                    current_frame(stackframes).push(return_value.clone());
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    NEW => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let class_index = &read_u16(&code.opcodes, pc);
                        let class_name_index = classdef.cp_class_ref(class_index);
                        let class_name = classdef.cp_utf8(class_name_index);
                        classmanager::load_class_by_name(class_name);
                        let class_to_instantiate = classmanager::get_class_by_name(class_name).unwrap();

                        let object = ObjectRef::Object(Rc::new(RefCell::new(Vm::new_instance(
                            class_to_instantiate,
                        ))));
                        current_frame(stackframes).push(Ref(object));
                    }
                    NEWARRAY => {
                        let arraytype = read_u8(&code.opcodes, pc);
                        let count = current_frame(stackframes).pop()?;
                        let array = ObjectRef::new_array(arraytype, count.into_i32() as usize);
                        current_frame(stackframes).push(Ref(array));
                    }
                    ANEWARRAY => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let class_index = &read_u16(&code.opcodes, pc);
                        let class_name_index = classdef.cp_class_ref(class_index);
                        let class_name = classdef.cp_utf8(class_name_index);
                        classmanager::load_class_by_name(class_name);
                        let arraytype = classmanager::get_class_by_name(class_name).unwrap();
                        let count = current_frame(stackframes).pop()?.into_i32();
                        let array = ObjectRef::new_object_array(arraytype, count as usize);
                        current_frame(stackframes).push(Ref(array));
                    }
                    ARRAYLENGTH => {
                        let val = current_frame(stackframes).pop()?;
                        if let Ref(val) = val {
                            current_frame(stackframes).push(I32(val.get_array_length() as i32));
                        } else {
                            unreachable!("array length {:?}", val);
                        }
                    }
                    MONITORENTER | MONITOREXIT => {
                        current_frame(stackframes).pop()?;
                    } //TODO implement
                    IFNULL | IFNONNULL => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3;
                        let value = current_frame(stackframes).pop()?;
                        let its_null = if let Null = value { true } else { false };

                        if its_null && opcode == IFNULL {
                            debug!("\t\tIF NULL =>{}: JMP {}", its_null, *pc + jmp_to as usize);
                            *pc += jmp_to as usize;
                        }
                        if !its_null && opcode == IFNONNULL {
                            debug!("\t\tIF NOT NULL =>{}: JMP {}", its_null, *pc + jmp_to as usize);
                            *pc += jmp_to as usize;
                        }
                        //debug info
                        if !its_null && opcode == IFNULL {
                            debug!("\t\tIF NULL =>false: NO JMP");
                        }
                        if its_null && opcode == IFNONNULL {
                            debug!("\t\tIF NONNULL =>false: NO JMP");
                        }
                    }
                    //TODO implement all opcodes
                    _ => {
                        panic!("opcode {} not implemented {:?}", opcode, stackframes)
                        //TODO implement proper --stacktraces-- error handling
                    }
                }
            }
        }
        panic!("should not happen")
    }

    fn if_cmp(pc: &mut usize, opcode: u8, jmp_to: u16, value1: &Value, value2: &Value) {
        if let I32(value1) = value1 {
            if let I32(value2) = value2 {
                let jump = match opcode {
                    IF_ICMPEQ => value1 == value2,
                    IF_ICMPNE => value1 != value2,
                    IF_ICMPGT => value1 > value2,
                    IF_ICMPGE => value1 >= value2,
                    IF_ICMPLT => value1 < value2,
                    IF_ICMPLE => value1 <= value2,
                    _ => false,
                };
                if jump {
                    debug!("\t\tIF({}) JMP {}", jump, *pc + jmp_to as usize);
                    *pc += jmp_to as usize;
                } else {
                    debug!("\t\tIF({}) NO JMP", jump);
                }
            }
        }
    }


    /// store in local param
    fn store(
        &mut self,
        stack: &mut Vec<StackFrame>,
        local_params: &mut Vec<Option<Value>>,
        index: usize,
    ) -> Result<(), Error> {
        let value = current_frame(stack).pop()?;
        while local_params.len() < index + 1 {
            local_params.push(None);
        }
        local_params[index] = Some(value);
        Ok(())
    }
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

pub(crate) fn current_frame(stackframes: &mut Vec<StackFrame>) -> &mut StackFrame {
    let i = stackframes.len() - 1;
    stackframes.get_mut(i).unwrap()
}