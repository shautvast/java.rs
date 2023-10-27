#![allow(non_snake_case)]

use log::info;
use once_cell::sync::Lazy;
use whoami::platform;

use crate::class::{unsafe_ref, unsafe_val, UnsafeValue, Value};
use crate::class::Value::Void;
use crate::heap::ObjectRef;

pub fn invoke_native(class_name: &String, method_name: &String, _args: Vec<UnsafeValue>) -> UnsafeValue {
    info!("native {}.{}", class_name, method_name);

    unsafe_val(match class_name.as_str() {
        "java/lang/Class" => java_lang_class(method_name),
        "jdk/internal/util/SystemProps$Raw" => jdk_internal_util_SystemProps_Raw(method_name),
        _ => Void
    })
}

fn java_lang_class(method_name: &String) -> Value {
    match method_name.as_str() {
        "desiredAssertionStatus0(Ljava/lang/Class;)Z" => Value::BOOL(false),
        _ => Void
    }
}

fn jdk_internal_util_SystemProps_Raw(method_name: &String) -> Value {
    match method_name.as_str() {
        "platformProperties()[Ljava/lang/String;" => systemProps(),
        "cmdProperties()Ljava/util/HashMap;" => cmdProps(), //TODO ability to instantiate classes here
        "vmProperties()[Ljava/lang/String;" => cmdProps(),
        _ => Void
    }
}

fn cmdProps() -> Value {
    Value::Null
}

fn systemProps() -> Value {
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
        Value::Ref(unsafe_ref(ObjectRef::StringArray(props.to_vec())))
    }
}