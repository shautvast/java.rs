use anyhow::{anyhow, Error};

use crate::class::ObjectRef;
use crate::class::Value::{self, *};


pub(crate) fn array_load(index: Value, arrayref: Value) -> Result<Value, Error> {
    if let I32(index) = index {
        let index = index as usize;

        if let Null = arrayref {
            return Err(anyhow!("NullpointerException"));
        }
        if let Ref(objectref) = arrayref {
            match objectref {
                ObjectRef::ByteArray(array) => {
                    return Ok(I32(array[index] as i32));
                }
                ObjectRef::ShortArray(array) => {
                    return Ok(I32(array[index] as i32));
                }
                ObjectRef::IntArray(array) => {
                    return Ok(I32(array[index]));
                }
                ObjectRef::BooleanArray(array) => {
                    return Ok(I32(array[index] as i32));
                }
                ObjectRef::CharArray(array) => {
                    return Ok(CHAR(array[index]));
                }
                ObjectRef::LongArray(array) => {
                    return Ok(I64(array[index]));
                }
                ObjectRef::FloatArray(array) => {
                    return Ok(F32(array[index]));
                }
                ObjectRef::DoubleArray(array) => {
                    return Ok(F64(array[index]));
                }
                ObjectRef::ObjectArray(_arraytype, data) => {
                    return Ok(Ref(data[index].clone()));
                }
                ObjectRef::StringArray(array) => {
                    return Ok(Utf8(array[index].to_owned()));
                }
                ObjectRef::Class(_) => {
                    panic!("should be array")
                }
                ObjectRef::Object(_) => {
                    panic!("should be array")
                } //throw error?
            }
        }
    }
    panic!()
}

pub(crate) fn array_store(value: Value, index: Value, arrayref: Value) -> Result<(), Error> {
    if let Null = arrayref {
        return Err(anyhow!("NullpointerException"));
    }

    if let I32(index) = index {
        if let Ref(mut objectref) = arrayref {
            match objectref {
                ObjectRef::ByteArray(ref mut array) => {
                    if let I32(value) = value {
                        // is i32 correct?
                        array[index as usize] = value as i8;
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::ShortArray(ref mut array) => {
                    if let I32(value) = value {
                        // is i32 correct?
                        array[index as usize] = value as i16;
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::IntArray(ref mut array) => {
                    if let I32(value) = value {
                        array[index as usize] = value;
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::BooleanArray(ref mut array) => {
                    if let I32(value) = value {
                        array[index as usize] = value > 0;
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::CharArray(ref mut array) => unsafe {
                    if let I32(value) = value {
                        array[index as usize] = char::from_u32_unchecked(value as u32);
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::LongArray(ref mut array) => {
                    if let I64(value) = value {
                        array[index as usize] = value;
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::FloatArray(ref mut array) => {
                    if let F32(value) = value {
                        array[index as usize] = value
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::DoubleArray(ref mut array) => {
                    if let F64(value) = value {
                        array[index as usize] = value
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::ObjectArray(_arraytype, ref mut array) => {
                    if let Ref(ref value) = value {
                        array[index as usize] = value.clone();
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::StringArray(ref mut array) => {
                    if let Utf8(ref value) = value {
                        array[index as usize] = value.clone();
                    } else {
                        unreachable!()
                    }
                }
                ObjectRef::Object(_) | ObjectRef::Class(_) => {} //throw error?
            }
        }
    } else {
        unreachable!()
    }
    Ok(())
}