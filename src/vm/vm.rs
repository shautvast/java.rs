use std::io::Write;

use anyhow::Error;
use log::{debug, info};

use crate::class::{Class, ClassId, Object, ObjectRef, Value};
use crate::class::Value::{F32, F64, I32, I64, Null, Ref, Utf8, Void};
use crate::classloader::classdef::{AttributeType, CpEntry, Method, Modifier};
use crate::classloader::io::{read_u16, read_u8};
use crate::classmanager;
use crate::vm::array::{array_load, array_store};
use crate::vm::native::invoke_native;
use crate::vm::opcodes;
use crate::vm::opcodes::*;
use crate::vm::operations::{get_signature_for_invoke, get_static};
use crate::vm::stack::StackFrame;

pub struct Vm {}


#[cfg(target_family = "unix")]
const PATH_SEPARATOR: char = ':';

#[cfg(target_family = "windows")]
const PATH_SEPARATOR: char = ';';

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
            .try_init();
        let mut vm_instance = Self {};
        classmanager::init();
        Vm::init(&mut vm_instance, stack);
        vm_instance
    }

    fn init(vm: &mut Vm, stack: &mut Vec<StackFrame>) {
        classmanager::load_class_by_name("java/lang/Class");
        Self::initialize_class(vm, stack, "java/lang/System");
    }

    fn initialize_class(vm: &mut Vm, stack: &mut Vec<StackFrame>, class: &str) {
        vm.execute_static(stack, class, "initPhase1()V", vec![]).expect("cannot create VM");
    }

    fn current_frame(stackframes: &mut Vec<StackFrame>) -> &mut StackFrame {
        let i = stackframes.len() - 1;
        stackframes.get_mut(i).unwrap()
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
                info!("arg {:?}",r);
            } else {
                info!("arg {:?}",arg);
            }
        }

        if let Null = args[0] {
            panic!("NPE");
        }
        if let Ref(this) = &args[0] {
            if let ObjectRef::Object(this) = this {
                let cd = classmanager::get_classdef(&this.class_id);
                let method = cd.get_method(method_name);
                if let Some(method) = method {
                    return self.execute_class_id(stack, this.class_id, &method, args.clone());
                } else {
                    let name = classmanager::classdef_name(&this.class_id);
                    if let Some(name) = name {
                        classmanager::load_class_by_name(&name);
                        let class = classmanager::get_class_by_name(&name).unwrap();
                        for parent_id in &class.parents {
                            if let Some(method) = method {
                                return self.execute_class(stack, class, method_name, args);
                            } else {
                                debug!("not {:?}", parent_id);
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
        let _classdef = classmanager::get_classdef(&class.id);
        self.execute_class(stack, class, method_name, args)
    }

    pub fn execute_class_id(&self, _stack: &mut Vec<StackFrame>, _this_class: ClassId, _method: &Method, _args: Vec<Value>) -> Result<Value, Error> {
        Ok(Null)
    }

    pub fn execute_class(
        &mut self,
        stackframes: &mut Vec<StackFrame>,
        this_class: &Class,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        info!("execute {}.{}", this_class.name, method_name);

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
                let cur_frame = Self::current_frame(stackframes);
                info!("\t{} #{} {} - {}", &cur_frame.at, &*pc - 1, opcodes::OPCODES[opcode as usize], cur_frame.len());
                match opcode {
                    ACONST_NULL => {
                        Self::current_frame(stackframes).push(Value::Null);
                    }
                    ICONST_M1 => {
                        Self::current_frame(stackframes).push(I32(-1));
                    }
                    ICONST_0 => {
                        Self::current_frame(stackframes).push(I32(0));
                    }
                    ICONST_1 => {
                        Self::current_frame(stackframes).push(I32(1));
                    }
                    ICONST_2 => {
                        Self::current_frame(stackframes).push(I32(2));
                    }
                    ICONST_3 => {
                        Self::current_frame(stackframes).push(I32(3));
                    }
                    ICONST_4 => {
                        Self::current_frame(stackframes).push(I32(4));
                    }
                    ICONST_5 => {
                        Self::current_frame(stackframes).push(I32(5));
                    }
                    LCONST_0 => {
                        Self::current_frame(stackframes).push(I64(0));
                    }
                    LCONST_1 => {
                        Self::current_frame(stackframes).push(I64(1));
                    }
                    FCONST_0 => {
                        Self::current_frame(stackframes).push(F32(0.0));
                    }
                    FCONST_1 => {
                        Self::current_frame(stackframes).push(F32(1.0));
                    }
                    FCONST_2 => {
                        Self::current_frame(stackframes).push(F32(2.0));
                    }
                    DCONST_0 => {
                        Self::current_frame(stackframes).push(F64(0.0));
                    }
                    DCONST_1 => {
                        Self::current_frame(stackframes).push(F64(1.0));
                    }
                    SIPUSH => {
                        let s = read_u16(&code.opcodes, pc) as i32;
                        Self::current_frame(stackframes).push(I32(s));
                    }
                    BIPUSH => {
                        let c = read_u8(&code.opcodes, pc) as i32;
                        Self::current_frame(stackframes).push(I32(c));
                    }
                    LDC => {
                        let cp_index = read_u8(&code.opcodes, pc) as u16;
                        let c = method.constant_pool.get(&cp_index).unwrap();
                        match c {
                            CpEntry::Integer(i) => {
                                Self::current_frame(stackframes).push(I32(*i));
                            }
                            CpEntry::Float(f) => {
                                Self::current_frame(stackframes).push(Value::F32(*f));
                            }
                            CpEntry::Double(d) => {
                                Self::current_frame(stackframes).push(Value::F64(*d));
                            }
                            CpEntry::StringRef(utf8) => {
                                //TODO
                                classmanager::load_class_by_name("java/lang/String");
                                let stringclass = classmanager::get_class_by_name("java/lang/String").unwrap();
                                let stringinstance =
                                    Ref(ObjectRef::Object(Vm::new_instance(stringclass)));

                                let string: Vec<u8> =
                                    classmanager::get_classdef(&this_class.id).cp_utf8(utf8)
                                        .to_owned()
                                        .as_bytes()
                                        .into();

                                self.execute_special(stackframes,
                                                     "java/lang/String",
                                                     "<init>([B)V",
                                                     vec![
                                                         stringinstance.clone(),
                                                         Ref(ObjectRef::new_byte_array(string)),
                                                     ],
                                )?;
                                Self::current_frame(stackframes).push(stringinstance);
                            }
                            CpEntry::Long(l) => {
                                Self::current_frame(stackframes).push(Value::I64(*l));
                            }
                            CpEntry::ClassRef(utf8) => {
                                let classdef = classmanager::get_classdef(&this_class.id);
                                let class_name = classdef.cp_utf8(utf8);
                                classmanager::load_class_by_name(class_name);
                                let klass_id = classmanager::get_classid(class_name);
                                if let Some(class) = classmanager::get_classobject(klass_id) {
                                    Self::current_frame(stackframes).push(class.clone());
                                } else {
                                    unreachable!("should not be here");
                                }
                            }
                            _ => {
                                panic!("add variant {:?}", c)
                            }
                        }
                    }
                    LDC_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        let cp_entry = method.constant_pool.get(&cp_index).unwrap();
                        match cp_entry {
                            CpEntry::Integer(i) => {
                                Self::current_frame(stackframes).push(I32(*i));
                            }
                            CpEntry::Float(f) => {
                                Self::current_frame(stackframes).push(F32(*f));
                            }
                            CpEntry::StringRef(utf8_index) => {
                                if let CpEntry::Utf8(s) = method.constant_pool.get(utf8_index).unwrap() {
                                    Self::current_frame(stackframes).push(Utf8(s.to_owned()));
                                } else {}
                            }
                            _ => {
                                println!("{:?}", cp_entry);
                                unreachable!()
                            }
                        }
                    }
                    LDC2_W => {
                        let cp_index = read_u16(&code.opcodes, pc);
                        match method.constant_pool.get(&cp_index).unwrap() {
                            CpEntry::Double(d) => {
                                Self::current_frame(stackframes).push(Value::F64(*d));
                            }
                            CpEntry::Long(l) => {
                                Self::current_frame(stackframes).push(Value::I64(*l));
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    ILOAD | LLOAD | FLOAD | DLOAD | ALOAD => {
                        // omitting the type checks so far
                        let n = read_u8(&code.opcodes, pc) as usize;
                        Self::current_frame(stackframes)
                            .push(local_params[n].as_ref().unwrap().clone());
                    }
                    ILOAD_0 | LLOAD_0 | FLOAD_0 | DLOAD_0 | ALOAD_0 => {
                        Self::current_frame(stackframes)
                            .push(local_params[0].as_ref().unwrap().clone());
                    }
                    ILOAD_1 | LLOAD_1 | FLOAD_1 | DLOAD_1 | ALOAD_1 => {
                        Self::current_frame(stackframes)
                            .push(local_params[1].as_ref().unwrap().clone());
                    }
                    ILOAD_2 | LLOAD_2 | FLOAD_2 | DLOAD_2 | ALOAD_2 => {
                        Self::current_frame(stackframes)
                            .push(local_params[2].as_ref().unwrap().clone());
                    }
                    ILOAD_3 | LLOAD_3 | FLOAD_3 | DLOAD_3 | ALOAD_3 => {
                        Self::current_frame(stackframes)
                            .push(local_params[3].as_ref().unwrap().clone());
                    }
                    IALOAD | LALOAD | FALOAD | DALOAD | AALOAD | BALOAD | CALOAD | SALOAD => {
                        let index = Self::current_frame(stackframes).pop()?;
                        let arrayref = Self::current_frame(stackframes).pop()?;
                        Self::current_frame(stackframes).push(array_load(index, arrayref)?);
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
                        let value = Self::current_frame(stackframes).pop()?;
                        let index = Self::current_frame(stackframes).pop()?;
                        let arrayref = Self::current_frame(stackframes).pop()?;
                        array_store(value, index, arrayref)?
                    }
                    POP => {
                        Self::current_frame(stackframes).pop()?;
                    }
                    DUP => {
                        let value = Self::current_frame(stackframes).pop()?;
                        Self::current_frame(stackframes).push(value.clone());
                        Self::current_frame(stackframes).push(value);
                    }
                    IDIV => {
                        let value1 = Self::current_frame(stackframes).pop()?;
                        let value2 = Self::current_frame(stackframes).pop()?;
                        Self::current_frame(stackframes).push(I32(value1.into_i32() / value2.into_i32()));
                    }

                    IFEQ | IFNE | IFLT | IFGE | IFGT | IFLE => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3; // -3 so that offset = location of Cmp opcode
                        let value = Self::current_frame(stackframes).pop()?;
                        Self::if_cmp(pc, opcode, jmp_to, &value, &I32(0));
                    }

                    IF_ICMPEQ | IF_ICMPNE | IF_ICMPGT | IF_ICMPGE | IF_ICMPLT | IF_ICMPLE => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3; // -3 so that offset = location of Cmp opcode
                        let value1 = Self::current_frame(stackframes).pop()?;
                        let value2 = Self::current_frame(stackframes).pop()?;
                        Self::if_cmp(pc, opcode, jmp_to, &value1, &value2);
                    }
                    GOTO => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3;
                        *pc += jmp_to as usize;
                    }
                    IRETURN | FRETURN | DRETURN | ARETURN => {
                        let result = Self::current_frame(stackframes).pop();
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

                        Self::current_frame(stackframes).push(field_value);
                    }
                    PUTSTATIC => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let cp_index = read_u16(&code.opcodes, pc);
                        let (class_index, field_name_and_type_index) =
                            classdef.cp_field_ref(&cp_index); // all these unwraps are safe as long as the class is valid
                        let (name_index, _) =
                            classdef.cp_name_and_type(field_name_and_type_index);
                        let name = classdef.cp_utf8(name_index);
                        let class_name_index = classdef.cp_class_ref(class_index);
                        let that_class_name = classdef.cp_utf8(class_name_index);

                        let val_index = if &this_class.name == that_class_name {
                            // may have to apply this in GETSTATIC too
                            this_class
                                .static_field_mapping
                                .get(that_class_name)
                                .unwrap()
                                .get(name)
                                .unwrap()
                                .index
                        } else {
                            classmanager::load_class_by_name(that_class_name);
                            let that = classmanager::get_class_by_name(that_class_name).unwrap();
                            that.static_field_mapping
                                .get(that_class_name)
                                .unwrap()
                                .get(name)
                                .unwrap()
                                .index
                        };
                        let value = Self::current_frame(stackframes).pop()?;
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

                        let objectref = Self::current_frame(stackframes).pop()?;
                        if let Ref(instance) = objectref {
                            if let ObjectRef::Object(mut object) = instance {
                                let value = object.get(this_class, declared_type, field_name);
                                Self::current_frame(stackframes).push(value.clone());
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

                        let value = Self::current_frame(stackframes).pop()?;
                        let objectref = Self::current_frame(stackframes).pop()?;
                        if let Ref(instance) = objectref {
                            if let ObjectRef::Object(mut object) = instance {
                                object.set(this_class, declared_type, field_name, value);
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
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, Self::current_frame(stackframes).pop()?.clone());
                            }
                            args.insert(0, Self::current_frame(stackframes).pop()?);
                            let return_value = self.execute_special(stackframes,
                                                                    &invocation.class_name,
                                                                    &invocation.method.name,
                                                                    args,
                            )?;
                            if let Ref(objectref) = &return_value {
                                if let ObjectRef::Object(object) = objectref {
                                    info!("return {:?}", object);
                                }
                            } else {
                                info!("return {:?}", return_value);
                            }
                            match return_value {
                                Void => {}
                                _ => {
                                    Self::current_frame(stackframes).push(return_value.clone());
                                }
                            }
                            // println!("stack {} at {}", Self::current_frame(stack).len(), Self::current_frame(stack).at)
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
                            let mut args = Vec::with_capacity(invocation.method.num_args);
                            for _ in 0..invocation.method.num_args {
                                args.insert(0, Self::current_frame(stackframes).pop()?.clone());
                            }
                            args.insert(0, Self::current_frame(stackframes).pop()?);
                            let return_value = self.execute_virtual(
                                stackframes,
                                &invocation.class_name,
                                &invocation.method.name,
                                args,
                            )?;
                            if let Ref(objectref) = &return_value {
                                if let ObjectRef::Object(object) = objectref {
                                    info!("return {:?}", object);
                                }
                            } else {
                                info!("return {:?}", return_value);
                            }
                            match return_value {
                                Void => {}
                                _ => {
                                    Self::current_frame(stackframes).push(return_value.clone());
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
                                args.insert(0, Self::current_frame(stackframes).pop()?.clone());
                            }
                            let return_value = self.execute_static(stackframes,
                                                                   &invocation.class_name,
                                                                   &invocation.method.name,
                                                                   args,
                            )?;
                            if let Ref(objectref) = &return_value {
                                if let ObjectRef::Object(object) = objectref {
                                    info!("return {:?}", object);
                                }
                            } else {
                                info!("return {:?}", return_value);
                            }
                            match return_value {
                                Void => {}
                                _ => {
                                    Self::current_frame(stackframes).push(return_value.clone());
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

                        let object = ObjectRef::Object(Vm::new_instance(
                            class_to_instantiate,
                        ));
                        Self::current_frame(stackframes).push(Ref(object));
                        // self.heap.new_object(object);
                    }
                    ANEWARRAY => {
                        let classdef = classmanager::get_classdef(&this_class.id);
                        let class_index = &read_u16(&code.opcodes, pc);
                        let class_name_index = classdef.cp_class_ref(class_index);
                        let class_name = classdef.cp_utf8(class_name_index);
                        classmanager::load_class_by_name(class_name);
                        let arraytype = classmanager::get_class_by_name(class_name).unwrap();
                        let count = Self::current_frame(stackframes).pop()?;
                        if let I32(count) = count {
                            let array = ObjectRef::new_object_array(arraytype, count as usize);

                            Self::current_frame(stackframes).push(Value::Ref(array));
                        } else {
                            panic!();
                        }
                    }
                    ARRAYLENGTH => {
                        let val = Self::current_frame(stackframes).pop()?;
                        if let Ref(val) = val {
                            Self::current_frame(stackframes).push(I32(val.get_array_length() as i32));
                        }
                    }
                    MONITORENTER => {
                        Self::current_frame(stackframes).pop()?;
                    } //TODO implement
                    IFNULL | IFNONNULL => {
                        let jmp_to = read_u16(&code.opcodes, pc) - 3;
                        let value = Self::current_frame(stackframes).pop()?;
                        let its_null = if let Null = value { true } else { false };

                        if its_null && opcode == IFNULL {
                            info!("\t\tIF NULL =>{}: JMP {}", its_null, *pc + jmp_to as usize);
                            *pc += jmp_to as usize;
                        }
                        if !its_null && opcode == IFNONNULL {
                            info!("\t\tIF NOT NULL =>{}: JMP {}", its_null, *pc + jmp_to as usize);
                            *pc += jmp_to as usize;
                        }
                        //debug info
                        if !its_null && opcode == IFNULL {
                            info!("\t\tIF NULL =>false: NO JMP");
                        }
                        if its_null && opcode == IFNONNULL {
                            info!("\t\tIF NONNULL =>false: NO JMP");
                        }
                    },
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
                    info!("\t\tIF({}) JMP {}", jump, *pc + jmp_to as usize);
                    *pc += jmp_to as usize;
                } else {
                    info!("\t\tIF({}) NO JMP", jump);
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
        let value = Self::current_frame(stack).pop()?;
        while local_params.len() < index + 1 {
            local_params.push(None);
        }
        local_params[index] = Some(value);
        Ok(())
    }
}

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

