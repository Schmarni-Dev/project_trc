pub fn string_to_color_hex_code(string: &str) -> String {
    let mut hash = 0;
    string.chars().for_each(|c| {
        let mut test: [u8; 4] = [0, 0, 0, 0];
        c.encode_utf8(&mut test);
        hash = u32::from_be_bytes(test) + ((hash << 5) - hash);
    });
    let mut color_str = "#".to_owned();
    (0..3).for_each(|i| {
        let value = (hash >> (i * 8)) & 0xff;
        color_str += &format!("{:02X}", value);
    });
    color_str
}
