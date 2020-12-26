use std::collections::HashMap;

#[macro_escape]
macro_rules! hashmap {
    ( $( $key:expr => $value:expr ),* ) => {
        {
            let mut hm = HashMap::new();
            $(hm.insert($key, $value);)*
            hm
        }
    };
}

#[test]
fn test_hashmap() {
    let empty_hm: HashMap<&str, &str> = hashmap!();
    assert!(empty_hm.len() == 0);
}
