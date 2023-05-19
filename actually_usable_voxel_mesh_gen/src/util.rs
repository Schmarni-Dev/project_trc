use std::{
    collections::hash_map::DefaultHasher,
    // default::default,
    fmt::Debug,
    hash::{Hash, Hasher},
    println,
};

pub fn string_to_color(string: &str) -> [u8; 4] {
    let mut s = DefaultHasher::new();
    string.hash(&mut s);

    let data: [u8; 4] = match s.finish().to_be_bytes() {
        [_, a, b, .., c, d] => [a.to_owned(), b.to_owned(), c.to_owned(), d.to_owned()],
    };
    data
}

#[allow(dead_code)]
pub fn debug_println(val: impl Debug) {
    println!("{:#?}", val)
}
