#![allow(non_snake_case)]

use log::info;
use once_cell::sync::Lazy;

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
            let mut vec = Vec::new();
            //TODO set values
            vec.push("display_country");
            vec.push("display_language");
            vec.push("display_script");
            vec.push("display_variant");
            vec.push("file_encoding");
            vec.push("file_separator");
            vec.push("format_country");
            vec.push("format_language");
            vec.push("format_script");
            vec.push("format_variant");
            vec.push("ftp_nonProxyHosts");
            vec.push("ftp_proxyHost");
            vec.push("ftp_proxyPort");
            vec.push("http_nonProxyHosts");
            vec.push("http_proxyHost");
            vec.push("http_proxyPort");
            vec.push("https_proxyHost");
            vec.push("https_proxyPort");
            vec.push("java_io_tmpdir");
            vec.push("line_separator");
            vec.push("os_arch");
            vec.push("os_name");
            vec.push("os_version");
            vec.push("path_separator");
            vec.push("socksNonProxyHosts");
            vec.push("socksProxyHost");
            vec.push("socksProxyPort");
            vec.push("stderr_encoding");
            vec.push("stdout_encoding");
            vec.push("sun_arch_abi");
            vec.push("sun_arch_data_model");
            vec.push("sun_cpu_endian");
            vec.push("sun_cpu_isalist");
            vec.push("sun_io_unicode_encoding");
            vec.push("sun_jnu_encoding");
            vec.push("sun_os_patch_level");
            vec.push("user_dir");
            vec.push("user_home");
            vec.push("user_name");
            vec.push("FIXED_LENGTH");

            vec.into_iter().map(|s| s.to_owned()).collect()
        });
        Value::Ref(unsafe_ref(ObjectRef::StringArray(props.to_vec())))
    }
}