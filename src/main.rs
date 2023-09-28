use regex::Regex;

fn main() {
    // if let Some(class) = classfile_reader::get_class(classfile_reader::io::read_class_file("./Dummy.class")){
    //     println!("{:?}", class);
    //     let ret = class.execute("public static get()D");
    //     println!("{:?}", ret);
    // }

    let pattern = Regex::new(".*/(.+)").unwrap();
    let c = pattern.captures("java/lang/String").unwrap().get(1);
    println!("{}", c.unwrap().as_str());
}


