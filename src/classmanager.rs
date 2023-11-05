use std::collections::{HashMap, LinkedList};
use once_cell::sync::Lazy;

use crate::class::{Class, ClassId, ObjectRef, TypeIndex, Value::*, Value};
use crate::class::Object;
use crate::classloader;
use crate::classloader::classdef::{ClassDef, Modifier};
use crate::classloader::io::PATH_SEPARATOR;
use crate::vm::Vm;

static mut CLASSMANAGER: Lazy<ClassManager> = Lazy::new(|| ClassManager::new());

pub fn init() {
    unsafe {
        CLASSMANAGER.classes.clear();
        CLASSMANAGER.names.clear();
        CLASSMANAGER.classdefs.clear();
        CLASSMANAGER.class_objects.clear();
        CLASSMANAGER.static_class_data.clear();
    }
}

pub fn set_classpath(classpath: &str) {
    unsafe {
        CLASSMANAGER.set_classpath(classpath);
    }
}

pub fn get_class_by_id(id: ClassId) -> Option<&'static Class> {
    unsafe {
        CLASSMANAGER.get_class_by_id(id)
    }
}

pub fn classdef_name(id: &ClassId) -> Option<String> {
    unsafe {
        CLASSMANAGER.classdef_name(id)
    }
}

pub fn get_classid(name: &str) -> &ClassId {
    unsafe {
        CLASSMANAGER.get_classid(name)
    }
}

pub fn get_classdef(id: &ClassId) -> &ClassDef {
    unsafe {
        CLASSMANAGER.get_classdef(id)
    }
}

pub fn load_class_by_name(name: &str) {
    unsafe {
        CLASSMANAGER.load_class_by_name(name)
    }
}

pub fn get_class_by_name(name: &str) -> Option<&Class> {
    unsafe {
        CLASSMANAGER.get_class_by_name(name)
    }
}

pub fn add_class(name: &str) -> ClassId {
    unsafe {
        CLASSMANAGER.add_class(name)
    }
}

pub fn get_static(id: &ClassId, index: usize) -> Value {
    unsafe {
        CLASSMANAGER.static_class_data.get(id).unwrap()[index].clone()
    }
}

pub fn set_static(id: &ClassId, index: usize, value: Value) {
    unsafe {
        CLASSMANAGER.static_class_data.get_mut(id).unwrap()[index] = value;
    }
}

pub fn get_classobject(id: &ClassId) -> Option<&Value> {
    unsafe {
        CLASSMANAGER.class_objects.get(id)
    }
}

//TODO less pubs
pub struct ClassManager {
    static_class_data: HashMap<ClassId, Vec<Value>>,
    // sequence for passing new classIds
    current_id: ClassId,
    // the classpath
    classpath: Vec<String>,

    //references to classdefs, ie the static class info
    pub classdefs: HashMap<ClassId, ClassDef>,

    // references to the runtime class
    pub classes: HashMap<ClassId, Class>,

    pub names: HashMap<String, ClassId>,
    pub class_objects: HashMap<ClassId, Value>,
    vm: Vm,
}

impl ClassManager {
    pub fn new() -> Self {
        Self {
            static_class_data: HashMap::new(),
            current_id: 0,
            classdefs: HashMap::new(),
            classes: HashMap::new(),
            class_objects: HashMap::new(),
            names: HashMap::new(),
            classpath: vec![],
            vm:Vm::new_internal(),
        }
    }

    fn set_classpath(&mut self, classpath: &str) {
        self.classpath = classpath
            .split(PATH_SEPARATOR)
            .map(|s| s.into())
            .collect();
    }

    fn get_class_by_id(&mut self, id: ClassId) -> Option<&Class> {
        if !self.classes.contains_key(&id) {
            let name = self.classdef_name(&id);
            if name.is_some() {
                self.add_class(&name.unwrap());
            }
        }
        self.classes.get(&id)
    }

    fn classdef_name(&self, id: &ClassId) -> Option<String> {
        self.classdefs.get(id).map(|c| c.name().to_owned()) //drops borrow to self here
    }

    fn get_classid(&self, name: &str) -> &ClassId {
        self.names.get(name).unwrap()
    }

    fn get_classdef(&self, id: &ClassId) -> &ClassDef {
        self.classdefs.get(&id).unwrap()
    }

    fn load_class_by_name(&mut self, name: &str) {
        let id = self.names.get(name);
        match id {
            Some(id) => if self.classes.get(id).is_none() {
                self.add_class(name);
            }
            None => { self.add_class(name); }
        }
    }

    fn get_class_by_name(&self, name: &str) -> Option<&Class> {
        let id = self.names.get(name);
        self.classes.get(id.unwrap())
    }

    fn add_class(&mut self, name: &str) -> ClassId {
        let this_classid = self.load(name);
        let this_classdef = self.classdefs.get(&this_classid).unwrap();

        //compute indices to fields
        let mut object_field_mapping = HashMap::new();
        let mut static_field_mapping = HashMap::new();
        let object_field_map_index: &mut usize = &mut 0;
        let static_field_map_index: &mut usize = &mut 0;

        let mut current_id = Some(this_classid);
        let mut current_classdef;
        let mut parents = LinkedList::new();
        while let Some(c) = current_id {
            parents.push_front(current_id.unwrap());
            current_classdef = self.classdefs.get(&c).unwrap();
            Self::add_fields_for_this_or_parents(&mut object_field_mapping, &mut static_field_mapping, object_field_map_index, static_field_map_index, current_classdef);

            current_id = current_classdef.super_class.as_ref()
                .map(|i| current_classdef.cp_class_name(i).to_owned())
                .map(|n| *self.names.get(&n).unwrap());
        }

        //handrolled references to superclass and interfaces
        let superclass_id = this_classdef.super_class.as_ref()
            .map(|i| this_classdef.cp_class_name(i).to_owned())
            .map(|n| *self.names.get(&n).unwrap());

        let interface_ids: Vec<ClassId> = this_classdef.interfaces.iter()
            .map(|i| this_classdef.cp_class_name(i).to_owned())
            .map(|n| *self.names.get(n.as_str()).unwrap())
            .collect();

        self.static_class_data.insert(this_classid, Self::set_field_data(&static_field_mapping));

        self.names.get(name)
            .and_then(|id|
                self.classes.insert(*id, Class {
                    id: *id,
                    initialized: false,
                    name: name.into(),
                    superclass: superclass_id,
                    parents,
                    interfaces: interface_ids,
                    object_field_mapping,
                    static_field_mapping,
                    // static_field_data: static_values,
                }));
        if name != "java/lang/Class" {
            let cls = self.get_class_by_name("java/lang/Class").unwrap();
            let mut instance = Object::new(cls);
            instance.set(cls, "java/lang/Class", "name", Value::Utf8(name.into()));
            let instance = Ref(ObjectRef::Object(instance));

            self.class_objects.insert(this_classid, instance);
        }
        let clinit = this_classdef.methods.contains_key("<clinit>()V");

        if clinit {
            self.vm.execute_special(&mut vec![],name, "<clinit>()V", vec![]).unwrap();
        }

        this_classid
    }

    fn add_fields_for_this_or_parents(object_field_mapping: &mut HashMap<String, HashMap<String, TypeIndex>>,
                                      static_field_mapping: &mut HashMap<String, HashMap<String, TypeIndex>>,
                                      object_field_map_index: &mut usize, static_field_map_index:
                                      &mut usize,
                                      current_classdef: &ClassDef) {
        let mut instance_field_mappings: HashMap<String, TypeIndex> = HashMap::new();
        let mut static_field_mappings: HashMap<String, TypeIndex> = HashMap::new();
        for (field_name, field) in &current_classdef.fields {
            if !field.is(Modifier::Static) {
                instance_field_mappings.insert(field_name.to_owned(),
                                               TypeIndex::new(field.type_of().to_owned(), *object_field_map_index));
                *object_field_map_index += 1;
            } else {
                static_field_mappings.insert(field_name.to_owned(),
                                             TypeIndex::new(field.type_of().to_owned(), *static_field_map_index));
                *static_field_map_index += 1;
            }
        }
        object_field_mapping.insert(current_classdef.name().to_owned(), instance_field_mappings);
        static_field_mapping.insert(current_classdef.name().to_owned(), static_field_mappings);
    }

    /// loads the class and recursively its dependencies
    fn load(&mut self, name: &str) -> ClassId {
        let (id, mut classes_to_load) = self.load_class_and_deps(name);
        while !classes_to_load.is_empty() {
            if let Some(classname) = classes_to_load.pop() {
                classes_to_load.append(&mut self.load_class_and_deps(classname.as_str()).1);
            }
        }
        id
    }

    /// loads the class and returns it's dependencies
    fn load_class_and_deps(&mut self, name: &str) -> (ClassId, Vec<String>) {
        let id = *self.names.entry(name.to_string()).or_insert_with(|| {
            self.current_id += 1;
            self.current_id
        });

        let classdef = self.classdefs
            .entry(id)
            .or_insert_with(|| classloader::get_classdef(&self.classpath, name).expect("ClassNotFound"));
        (self.current_id, inspect_dependencies(classdef))
    }

    pub(crate) fn set_field_data(field_mapping: &HashMap<String, HashMap<String, TypeIndex>>) -> Vec<Value> {
        let mut field_data = vec![Null; n_fields(field_mapping)];

        for (_, this_class) in field_mapping {
            for (_name, type_index) in this_class {
                let value = match type_index.type_name.as_str() {
                    "Z" => BOOL(false),
                    "B" => I32(0),
                    "S" => I32(0),
                    "I" => I32(0),
                    "J" => I64(0),
                    "F" => F32(0.0),
                    "D" => F64(0.0),
                    _ => Null,
                };
                // println!("{} = {:?}", name, value);
                field_data[type_index.index] = value.into();
            }
        }

        field_data
    }
}

pub(crate) fn n_fields(field_mapping: &HashMap<String, HashMap<String, TypeIndex>>) -> usize {
    field_mapping
        .iter()
        .map(|(_, v)| v.len())
        .reduce(|acc, e| acc + e)
        .unwrap()
}

pub(crate) fn inspect_dependencies(classdef: &ClassDef) -> Vec<String> {
    let mut classes_to_load: Vec<String> = vec![];

    if let Some(superclass) = &classdef.super_class {
        classes_to_load.push(classdef.cp_class_name(superclass).into());
    }
    for interface in &classdef.interfaces {
        classes_to_load.push(classdef.cp_class_name(interface).into());
    }
    classes_to_load
}