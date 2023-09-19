#[derive(Debug)]
//TODO create factory function
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<CpEntry>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}


#[derive(Debug)]
pub enum CpEntry {
    Utf8(String),
    //1
    Integer(i32),
    //3
    Float(f32),
    //4
    Long(i64),
    //5
    Double(f64),
    //6
    Class(u16),
    //7
    String(u16),
    //8
    Fieldref(u16, u16),
    //9
    MethodRef(u16, u16),
    //10
    InterfaceMethodref(u16, u16),
    //11
    NameAndType(u16, u16), //12
}

#[derive(Debug)]
pub struct Field {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub struct Attribute {
    pub attribute_name_index: u16,
    pub info: Vec<u8>,
}

#[derive(Debug)]
pub struct Method{
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<Attribute>,
}