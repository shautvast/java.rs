#![allow(non_snake_case)]

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Error;
use log::debug;
use once_cell::sync::Lazy;
use crate::classmanager::ClassManager;
use crate::value::Value;
use crate::value::Value::{I32, Void};
use crate::vm::object::ObjectRef;
use crate::vm::object::ObjectRef::Object;
use crate::vm::runtime::{Stackframe, Vm};


pub fn invoke_native(class_manager: &mut ClassManager, class_name: &str, method_name: &str, _args: Vec<Value>) -> Result<Value, Error> {
    debug!("native {}.{}", class_name, method_name);

    match class_name {
        "java/lang/Class" => java_lang_Class(method_name),
        "java/lang/System" => java_lang_System(method_name),
        "jdk/internal/misc/Unsafe" => jdk_internal_misc_Unsafe(method_name),
        "jdk/internal/util/SystemProps$Raw" => jdk_internal_util_SystemProps_Raw(class_manager, method_name),
        _ => unimplemented!("")
    }
}

fn java_lang_Class(method_name: &str) -> Result<Value, Error> {
    Ok(match method_name {
        "desiredAssertionStatus0(Ljava/lang/Class;)Z" => Value::BOOL(false),
        _ => Void
    })
}

fn java_lang_System(method_name: &str) -> Result<Value, Error> {
    Ok(match method_name {
        _ => Void
    })
}

fn jdk_internal_misc_Unsafe(method_name: &str) -> Result<Value, Error> {
    Ok(match method_name {
        "arrayBaseOffset0(Ljava/lang/Class;)I" => I32(0), //TODO surely this is not right
        "arrayIndexScale0(Ljava/lang/Class;)I" => I32(0), //TODO surely this is not right
        _ => Void
    })
}

fn jdk_internal_util_SystemProps_Raw(class_manager: &mut ClassManager, method_name: &str) -> Result<Value, Error> {
    match method_name {
        "platformProperties()[Ljava/lang/String;" => platformProperties(),
        "cmdProperties()Ljava/util/HashMap;" => cmdProps(class_manager), //TODO ability to instantiate classes here
        "vmProperties()[Ljava/lang/String;" => vmProperties(),
        _ => Ok(Void)
    }
}

fn cmdProps(class_manager: &mut ClassManager) -> Result<Value, Error> {
    class_manager.load_class_by_name("java/util/HashMap");
    let hashmap_class = class_manager.get_class_by_name("java/util/HashMap").unwrap();
    let hashmap = Value::Ref(Object(Rc::new(RefCell::new(crate::vm::object::Object::new(hashmap_class))))); // this is convoluted
    Stackframe::new(vec![hashmap.clone()]).run(class_manager, hashmap_class.id, "<init>()V");
    Ok(hashmap)
}

fn vmProperties() -> Result<Value, Error> {
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