use rust_irc::{
    buf_helpers::{
        one_op_buf, one_param_buf, three_param_buf, two_op_buf, two_param_buf, SPACE_BYTES,
    },
    codes,
};

#[test]
pub fn test_one_op_buf() {
    let buf_in: [u8; 1] = one_op_buf(codes::RESPONSE_OK);
    assert_eq!(buf_in, [codes::RESPONSE_OK])
}

#[test]
pub fn test_two_op_buf() {
    let buf_in: [u8; 2] = two_op_buf(codes::ERROR, codes::error::ALREADY_IN_ROOM);
    assert_eq!(buf_in, [codes::ERROR, codes::error::ALREADY_IN_ROOM]);
}

#[test]
pub fn test_one_param_buf() {
    let opcode_buf: &[u8; 1] = &[codes::MESSAGE];
    let string_buf: &[u8] = "hello world".as_bytes();
    let checker_buf: Vec<u8> = [opcode_buf, string_buf].concat();
    let result: Vec<u8> = one_param_buf(codes::MESSAGE, "hello world");
    assert_eq!(result, checker_buf);
}

#[test]
pub fn test_two_param_buf() {
    let opcode_buf: &[u8; 1] = &[codes::MESSAGE];
    let string0_buf: &[u8] = "cat".as_bytes();
    let string1_buf: &[u8] = "dog".as_bytes();

    let checker_buf: Vec<u8> = [opcode_buf, string0_buf, SPACE_BYTES, string1_buf].concat();
    let result: Vec<u8> = two_param_buf(codes::MESSAGE, "cat", "dog");
    assert_eq!(result, checker_buf);
}

#[test]
pub fn test_three_param_buf() {
    let opcode_buf: &[u8; 1] = &[codes::MESSAGE];
    let string0_buf: &[u8] = "cat".as_bytes();
    let string1_buf: &[u8] = "dog".as_bytes();
    let string2_buf: &[u8] = "frog".as_bytes();

    let checker_buf: Vec<u8> = [
        opcode_buf,
        string0_buf,
        SPACE_BYTES,
        string1_buf,
        SPACE_BYTES,
        string2_buf,
    ]
    .concat();
    let result: Vec<u8> = three_param_buf(codes::MESSAGE, "cat", "dog", "frog");
    assert_eq!(result, checker_buf);
}
