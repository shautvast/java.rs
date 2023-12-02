use crate::vm::object::ObjectRef::*;
use anyhow::{anyhow, Error};

use crate::value::Value;
use crate::value::Value::*;

pub(crate) fn array_load(index: Value, arrayref: Value) -> Result<Value, Error> {
    if let I32(index) = index {
        let index = index as usize;

        if let Null = arrayref {
            return Err(anyhow!("NullpointerException"));
        }
        if let Ref(objectref) = arrayref {
            match objectref {
                ByteArray(array) => {
                    return Ok(I32(array[index] as i32));
                }
                ShortArray(array) => {
                    return Ok(I32(array[index] as i32));
                }
                IntArray(array) => {
                    return Ok(I32(array[index]));
                }
                BooleanArray(array) => {
                    return Ok(I32(array[index] as i32));
                }
                CharArray(array) => {
                    return Ok(CHAR(array[index]));
                }
                LongArray(array) => {
                    return Ok(I64(array[index]));
                }
                FloatArray(array) => {
                    return Ok(F32(array[index]));
                }
                DoubleArray(array) => {
                    return Ok(F64(array[index]));
                }
                ObjectArray(_arraytype, data) => {
                    return Ok(Ref(data[index].clone()));
                }
                StringArray(array) => {
                    return Ok(Utf8(array[index].to_owned()));
                }
                Class(_) => {
                    panic!("should be array")
                }
                Object(_) => {
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
                ByteArray(ref mut array) => {
                    if let I32(value) = value {
                        // is i32 correct?
                        array[index as usize] = value as i8;
                    } else {
                        unreachable!()
                    }
                }
                ShortArray(ref mut array) => {
                    if let I32(value) = value {
                        // is i32 correct?
                        array[index as usize] = value as i16;
                    } else {
                        unreachable!()
                    }
                }
                IntArray(ref mut array) => {
                    if let I32(value) = value {
                        array[index as usize] = value;
                    } else {
                        unreachable!()
                    }
                }
                BooleanArray(ref mut array) => {
                    if let I32(value) = value {
                        array[index as usize] = value > 0;
                    } else {
                        unreachable!()
                    }
                }
                CharArray(ref mut array) => {
                    if let I32(value) = value {
                        array[index as usize] = char::from_u32(value as u32).unwrap();
                    } else {
                        unreachable!()
                    }
                },
                LongArray(ref mut array) => {
                    if let I64(value) = value {
                        array[index as usize] = value;
                    } else {
                        unreachable!()
                    }
                }
                FloatArray(ref mut array) => {
                    if let F32(value) = value {
                        array[index as usize] = value
                    } else {
                        unreachable!()
                    }
                }
                DoubleArray(ref mut array) => {
                    if let F64(value) = value {
                        array[index as usize] = value
                    } else {
                        unreachable!()
                    }
                }
                ObjectArray(_arraytype, ref mut array) => {
                    if let Ref(ref value) = value {
                        array[index as usize] = value.clone();
                    } else {
                        unreachable!()
                    }
                }
                StringArray(ref mut array) => {
                    if let Utf8(ref value) = value {
                        array[index as usize] = value.clone();
                    } else {
                        unreachable!()
                    }
                }
                Object(_) | Class(_) => {} //throw error?
            }
        }
    } else {
        unreachable!()
    }
    Ok(())
}
