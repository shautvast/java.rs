use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use crate::{CpEntry, opcodes};
use crate::io::read_u16;

#[derive(Debug)]
//TODO create factory function
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Rc<HashMap<usize, CpEntry>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: HashMap<String, Method>,
    pub attributes: HashMap<String, AttributeType>,
}

impl Class {
    pub fn get_version(&self) -> (u16, u16) {
        (self.major_version, self.minor_version)
    }

    pub fn execute(&self, method_name: &str) -> Value {
        let m = self.methods.get(method_name).unwrap();
        m.execute().unwrap() //TODO remove unwrap
    }
}

pub struct Method {
    constant_pool: Rc<HashMap<usize, CpEntry>>,
    access_flags: u16,
    name_index: usize,
    descriptor_index: usize,
    attributes: HashMap<String, AttributeType>,
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Method {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
               self.access_flags, self.name_index, self.descriptor_index, self.attributes)
    }
}

impl Method {
    pub fn new(constant_pool: Rc<HashMap<usize, CpEntry>>,
               access_flags: u16,
               name_index: usize,
               descriptor_index: usize,
               attributes: HashMap<String, AttributeType>, ) -> Self {
        Method { constant_pool, access_flags, name_index, descriptor_index, attributes }
    }

    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.name_index).unwrap() {
            full_name.push_str(s);
        }
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.descriptor_index).unwrap() {
            full_name.push_str(s);
        }


        full_name
    }

    pub fn execute(&self) -> Option<Value> {
        if let AttributeType::Code(code) = self.attributes.get("Code").unwrap() {
            let mut stack = Stack::new();
            let mut pc: usize = 0;
            while pc < code.opcodes.len() {
                let opcode = &code.opcodes[pc];
                pc += 1;
                println!("{}", opcode);
                match opcode {
                    &opcodes::bipush => {
                        let c = code.opcodes[pc] as i32;
                        stack.push(Value::I32(c));
                        pc += 1;
                    }
                    &opcodes::ldc2_w => {
                        let cp_index = read_u16(&code.opcodes, pc) as usize;
                        if let CpEntry::Double(d) = self.constant_pool.get(&cp_index).unwrap() {
                            stack.push(Value::F64(*d));
                        }
                        pc += 2;
                    }
                    &opcodes::ireturn => {
                        return stack.pop();
                    }
                    &opcodes::dreturn => {
                        return stack.pop();
                    }
                    //TODO implement all opcodes
                    _ => { panic!("opcode not implemented") }
                }

            }
        }
        None // TODO error situation
    }
}

pub struct Field {
    constant_pool: Rc<HashMap<usize, CpEntry>>,
    access_flags: u16,
    name_index: usize,
    descriptor_index: usize,
    attributes: HashMap<String, AttributeType>,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field {{access_flags: {}, name_index: {}, descriptor_index: {}, attributes: {:?} }}",
               self.access_flags, self.name_index, self.descriptor_index, self.attributes)
    }
}

impl Field {
    pub fn new(constant_pool: Rc<HashMap<usize, CpEntry>>,
               access_flags: u16,
               name_index: usize,
               descriptor_index: usize,
               attributes: HashMap<String, AttributeType>, ) -> Self {
        Field { constant_pool, access_flags, name_index, descriptor_index, attributes: attributes }
    }

    pub fn name(&self) -> String {
        let mut full_name = get_modifier(self.access_flags);

        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.descriptor_index).unwrap() {
            full_name.push_str(s);
        }
        full_name.push(' ');
        if let CpEntry::Utf8(s) = &self.constant_pool.get(&self.name_index).unwrap() {
            full_name.push_str(s);
        }

        full_name
    }
}

const MODIFIERS: [(u16, &str); 12] = [
    (0x0001, "public "),
    (0x0002, "private "),
    (0x0004, "protected "),
    (0x0008, "static "),
    (0x0010, "final "),
    (0x0020, "synchronized "),
    (0x0040, "volatile "),
    (0x0080, "transient "),
    (0x0100, "native "),
    (0x0200, "interface "),
    (0x0400, "interface "),
    (0x0800, "strict ")];

pub fn get_modifier(modifier: u16) -> String {
    let mut output = String::new();
    for m in MODIFIERS {
        if modifier & m.0 == m.0 { output.push_str(m.1) }
    }
    output
}


#[derive(Debug)]
pub enum AttributeType {
    ConstantValue(u16),
    Code(MethodCode),
    StackMapTable,
    BootstrapMethods,
    NestHost,
    NestMembers,
    PermittedSubclasses,
    Exceptions,
    InnerClasses,
    EnclosingMethod,
    Synthetic,
    Signature,
    Record,
    SourceFile,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    SourceDebugExtension,
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault,
    MethodParameters,
    Module,
    ModulePackages,
    ModuleMainClass,
}

#[derive(Debug)]
pub struct Exception {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

impl Exception {
    pub fn read(code: &[u8], index: usize) -> Self {
        Self {
            start_pc: read_u16(code, index),
            end_pc: read_u16(code, index + 2),
            handler_pc: read_u16(code, index + 4),
            catch_type: read_u16(code, index + 6),
        }
    }
}

#[derive(Debug)]
pub struct MethodCode {
    max_stack: u16,
    max_locals: u16,
    opcodes: Vec<u8>,
    exception_table: Vec<Exception>,
    code_attributes: HashMap<String, AttributeType>,
}

impl MethodCode {
    pub(crate) fn new(max_stack: u16, max_locals: u16,
                      code: Vec<u8>,
                      exception_table: Vec<Exception>,
                      code_attributes: HashMap<String, AttributeType>) -> Self {
        Self { max_stack, max_locals, opcodes: code, exception_table, code_attributes }
    }
}

struct Stack {
    data: Vec<Value>,
}

impl Stack {
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

#[derive(Debug)]
pub enum Value {
    Void,
    I32(i32),
    F64(f64),
}