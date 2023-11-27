mod test {
    use java_rs::classloader::classdef::{Method, Modifier};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn access_flags() {
        let m = Method::new(
            Rc::new(HashMap::new()),
            Modifier::Public as u16 | Modifier::Static as u16,
            0,
            0,
            HashMap::new(),
        );
        assert!(m.is(Modifier::Public));
        assert!(m.is(Modifier::Static));
        assert!(!m.is(Modifier::Private));
    }
}
