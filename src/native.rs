#![allow(non_snake_case)]

use std::ptr::hash;
use anyhow::Error;
use log::{debug, info};
use once_cell::sync::Lazy;

use crate::class::{get_class, unsafe_ref, Value};
use crate::class::Value::Void;
use crate::heap::ObjectRef;
use crate::heap::ObjectRef::Object;
use crate::vm::Vm;

pub fn invoke_native(vm: &mut Vm, class_name: &String, method_name: &String, _args: Vec<Value>) -> Result<Value,Error> {
    info!("native {}.{}", class_name, method_name);

    match class_name.as_str() {
        "java/lang/Class" => java_lang_class(vm, method_name),
        "jdk/internal/util/SystemProps$Raw" => jdk_internal_util_SystemProps_Raw(vm, method_name),
        _ => Ok(Void)
    }
}

fn java_lang_class(_vm: &mut Vm, method_name: &String) -> Result<Value,Error> {
    Ok(match method_name.as_str() {
        "desiredAssertionStatus0(Ljava/lang/Class;)Z" => Value::BOOL(false),
        _ => Void
    })
}

fn jdk_internal_util_SystemProps_Raw(vm: &mut Vm,method_name: &String) -> Result<Value,Error> {
    match method_name.as_str() {
        "platformProperties()[Ljava/lang/String;" => systemProps(),
        "cmdProperties()Ljava/util/HashMap;" => cmdProps(vm), //TODO ability to instantiate classes here
        "vmProperties()[Ljava/lang/String;" => cmdProps(vm),
        _ => Ok(Void)
    }
}

fn cmdProps(vm: &mut Vm,) -> Result<Value,Error> {
    let hashmap_class = get_class(vm, "java/util/HashMap")?;
    let hashmap = Vm::new_instance(hashmap_class);
    let hashmap = Value::Ref(unsafe_ref(Object(Box::new(hashmap))));
    vm.execute_special("java/util/HashMap", "<init>()V", vec![hashmap.clone()]);
    unsafe {debug!("hashmap {:?}", *hashmap.into_object().get());}
    panic!()
}

fn systemProps() -> Result<Value,Error> {
    unsafe {
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
            }else {
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
        Ok(Value::Ref(unsafe_ref(ObjectRef::StringArray(props.to_vec()))))
    }
}