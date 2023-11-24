use std::collections::{HashMap, LinkedList};

pub type ClassId = usize;

#[derive(Debug, Clone)]
pub(crate) struct TypeIndex {
    pub type_name: String,
    pub index: usize,
}

impl TypeIndex {
    pub(crate) fn new(type_name: String, index: usize) -> Self {
        Self {
            type_name,
            index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    pub id: ClassId,
    pub initialized: bool,
    pub name: String,
    pub superclass: Option<ClassId>,
    pub parents: LinkedList<ClassId>,
    pub interfaces: Vec<ClassId>,
    // lookup index and type from the name of the declared class and then field
    pub(crate) object_field_mapping: HashMap<String, HashMap<String, TypeIndex>>,
    pub(crate) static_field_mapping: HashMap<String, HashMap<String, TypeIndex>>,
    // pub(crate) static_field_data: Vec<Value> // moved to classmanager
}

impl Class {
    /// gets the number of non-static fields on the class
    pub(crate) fn n_object_fields(&self) -> usize {
        self.object_field_mapping
            .iter()
            .map(|(_, v)| v.len())
            .reduce(|acc, e| acc + e)
            .unwrap_or(0)
    }
}