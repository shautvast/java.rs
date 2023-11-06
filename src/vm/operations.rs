use std::collections::HashMap;

use anyhow::Error;
use log::debug;

use crate::class::{Class, Value};
use crate::classloader::classdef::CpEntry;
use crate::classmanager;
use crate::vm::vm::{Invocation, MethodSignature};

/// the place for opcode implementations that are a bit long

// GET_STATIC opcode
pub(crate) fn get_static(this_class: &Class, field_index: u16) -> Result<Value, Error> {
    let classdef = classmanager::get_classdef(&this_class.id);
    let (class_index, field_name_and_type_index) =
        classdef.cp_field_ref(&field_index); // all these unwraps are safe as long as the class is valid
    let (name_index, _) =
        classdef.cp_name_and_type(field_name_and_type_index);
    let field_name = classdef.cp_utf8(name_index);
    let that_class_name_index = classdef.cp_class_ref(class_index);
    let that_class_name = classdef.cp_utf8(that_class_name_index);
    classmanager::load_class_by_name(that_class_name);
    let that_class = classmanager::get_class_by_name(that_class_name).unwrap();

    let type_index = that_class
        .static_field_mapping
        .get(that_class_name)
        .unwrap()// safe because class for static field must be there
        .get(field_name)
        .unwrap(); // safe because field must be there

    debug!("get_static {}.{}", that_class_name, field_name);
    Ok(classmanager::get_static(&that_class.id, type_index.index))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn num_args() {
        let n = get_num_args("(Ljava/nio/charset/Charset;[BII)V");
        assert_eq!(n, 4)
    }
}