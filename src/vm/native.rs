#![allow(non_snake_case)]

use std::cell::RefCell;
use std::rc::Rc;
use anyhow::Error;
use log::debug;
use once_cell::sync::Lazy;

use crate::class::{ObjectRef, Value};
use crate::class::ObjectRef::Object;
use crate::class::Value::Void;
use crate::classmanager;
use crate::vm::stack::StackFrame;
use crate::vm::Vm;

pub fn invoke_native(vm: &mut Vm, stackframes: &mut Vec<StackFrame>, class_name: &String, method_name: &str, _args: Vec<Value>) -> Result<Value, Error> {
    debug!("native {}.{}", class_name, method_name);

    match class_name.as_str() {
        "java/lang/Class" => java_lang_class(vm, method_name),
        "jdk/internal/util/SystemProps$Raw" => jdk_internal_util_SystemProps_Raw(vm, stackframes, method_name),
        _ => Ok(Void)
    }
}

fn java_lang_class(_vm: &Vm, method_name: &str) -> Result<Value, Error> {
    Ok(match method_name {
        "desiredAssertionStatus0(Ljava/lang/Class;)Z" => Value::BOOL(false),
        _ => Void
    })
}

fn jdk_internal_util_SystemProps_Raw(vm: &mut Vm, stackframes: &mut Vec<StackFrame>, method_name: &str) -> Result<Value, Error> {
    match method_name {
        "platformProperties()[Ljava/lang/String;" => platformProperties(),
        "cmdProperties()Ljava/util/HashMap;" => cmdProps(vm, stackframes), //TODO ability to instantiate classes here
        "vmProperties()[Ljava/lang/String;" => vmProperties(vm, stackframes),
        _ => Ok(Void)
    }
}

fn cmdProps(vm: &mut Vm, stackframes: &mut Vec<StackFrame>) -> Result<Value, Error> {
    classmanager::load_class_by_name("java/util/HashMap");
    let hashmap_class = classmanager::get_class_by_name("java/util/HashMap").unwrap();
    let hashmap = Vm::new_instance(hashmap_class);
    let hashmap = Value::Ref(Object(Rc::new(RefCell::new(hashmap))));
    vm.execute_special(stackframes, "java/util/HashMap", "<init>()V", vec![hashmap.clone()])?;
    Ok(hashmap)
}

fn vmProperties(_vm: &mut Vm, _stackframes: &mut Vec<StackFrame>) -> Result<Value, Error> {
    let props: Lazy<Vec<String>> = Lazy::new(|| {
        let vec: Vec<String> = Vec::new();
        //TODO insert some values
        vec
    });
    Ok(Value::Ref(ObjectRef::StringArray(props.to_vec())))
}

fn platformProperties() -> Result<Value, Error> {
    let props: Lazy<Vec<String>> = Lazy::new(|| {
        let mut vec: Vec<String> = Vec::new();
        //TODO set correct values
        vec.push("display_country".into()); //null in jdk21
        vec.push("display_language".into()); //null in jdk21
        vec.push("display_script".into()); //null in jdk21
        vec.push("display_variant".into()); //null in jdk21
        vec.push("UTF-8".into());

        {
            #[cfg(target_family = "unix")]
            vec.push("/".into());
            #[cfg(target_family = "windows")]
            vec.push("\\");
        }
        vec.push("format_country".into()); //null in jdk21
        vec.push("format_language".into()); //null in jdk21
        vec.push("format_script".into()); //null in jdk21
        vec.push("format_variant".into()); //null in jdk21
        vec.push("ftp_nonProxyHosts".into());
        if let Ok(ftp_proxy) = std::env::var("ftp_proxy") {
            vec.push(ftp_proxy.to_owned());//TODO
            vec.push(ftp_proxy);
        } else {
            vec.push("".to_owned());
            vec.push("".to_owned());
        }

        vec.push("http_nonProxyHosts".into());
        if let Ok(http_proxy) = std::env::var("http_proxy") {
            vec.push(http_proxy.to_owned());
            vec.push(http_proxy);//TODO
        } else {
            vec.push("".to_owned());
            vec.push("".to_owned());
        }
        if let Ok(https_proxy) = std::env::var("https_proxy") {
            vec.push(https_proxy.to_owned());
            vec.push(https_proxy);
        } else {
            vec.push("".to_owned());
            vec.push("".to_owned());
        }
        vec.push(std::env::temp_dir().display().to_string());

        {
            #[cfg(target_family = "unix")]
            vec.push("\n".into());
            #[cfg(target_family = "windows")]
            vec.push("\r\n");
        }
        vec.push(whoami::platform().to_string());
        vec.push(whoami::devicename());
        vec.push("os_version".into());
        {
            #[cfg(target_family = "unix")]
            vec.push(":".into());
            #[cfg(target_family = "windows")]
            vec.push(";".into());
        }
        vec.push("socksNonProxyHosts".into());
        vec.push("socksProxyHost".into());
        vec.push("socksProxyPort".into());
        vec.push("UTF-8".into());
        vec.push("UTF-8".into());
        vec.push("sun_arch_abi".into());
        vec.push("sun_arch_data_model".into());
        vec.push("sun_cpu_endian".into()); //null in jdk21
        vec.push("sun_cpu_isalist".into()); //null in jdk21
        vec.push("sun_io_unicode_encoding".into()); //null in jdk21
        vec.push("sun_jnu_encoding".into()); //null in jdk21
        vec.push("sun_os_patch_level".into()); //null in jdk21
        if let Ok(curdir) = std::env::current_dir() {
            vec.push(curdir.display().to_string());
        }

        let home = std::env::home_dir().unwrap();
        vec.push(home.display().to_string());
        vec.push(whoami::username());
        vec.push("FIXED_LENGTH".into());

        vec
    });
    Ok(Value::Ref(ObjectRef::StringArray(props.to_vec())))
}