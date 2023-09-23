pub(crate) fn read_u16(data: &[u8], pos: usize) -> u16 {
    u16::from_be_bytes(data[pos..pos + 2].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_i32(data: &[u8], pos: usize) -> i32 {
    i32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_u32(data: &[u8], pos: usize) -> u32 {
    u32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_f32(data: &[u8], pos: usize) -> f32 {
    f32::from_be_bytes(data[pos..pos + 4].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_i64(data: &[u8], pos: usize) -> i64 {
    i64::from_be_bytes(data[pos..pos + 8].try_into().expect("slice with incorrect length"))
}

pub(crate) fn read_f64(data: &[u8], pos: usize) -> f64 {
    f64::from_be_bytes(data[pos..pos + 8].try_into().expect("slice with incorrect length"))
}