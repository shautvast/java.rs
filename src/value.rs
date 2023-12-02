use crate::vm::object::ObjectRef;

#[derive(Debug, Clone)]
pub enum Value {
    // variant returned for void methods
    Void,
    // 'pointer' to nothing
    Null,
    // primitives
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BOOL(bool),
    CHAR(char),
    // objects and arrays
    Ref(ObjectRef),
    // special object
    Utf8(String),
    F32Infinity,
    F32Nan,
    F64Infinity,
    F64Nan
}

impl Value {
    // panics if not correct type
    pub fn into_i32(self) -> i32 {
        if let Value::I32(v) = self {
            v
        } else {
            panic!("{:?} is not I32", self);
        }
    }

    pub fn into_i64(self) -> i64 {
        if let Value::I64(v) = self {
            v
        } else {
            panic!("{:?} is not I64", self);
        }
    }

    pub fn into_f32(self) -> f32 {
        if let Value::F32(v) = self {
            v
        } else {
            panic!("{:?} is not F32", self);
        }
    }

    pub fn into_f64(self) -> f64 {
        if let Value::F64(v) = self {
            v
        } else {
            panic!("{:?} is not F64", self);
        }
    }

    pub fn into_object(self) -> ObjectRef {
        if let Value::Ref(v) = self {
            v
        } else {
            panic!();
        }
    }
}
