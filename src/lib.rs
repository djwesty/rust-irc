pub mod codes {
    pub const TBD: u8 = 0x00;
    pub const JOIN_ROOM: u8 = 0x01;
    pub const JOIN_SERVER: u8 = 0x02;
    pub const LEAVE_ROOM: u8 = 0x03;
    pub const LIST_ROOMS: u8 = 0x04;
    pub const MESSAGE: u8 = 0x05;
    pub const REGISTER_NICK: u8 = 0x06;
    pub const LIST_USERS: u8 = 0x07;
    pub const LIST_USERS_IN_ROOM: u8 = 0x08;
    pub const MESSAGE_ROOM: u8 = 0x09;
    pub const QUIT: u8 = 0x0B;
    pub const KEEP_ALIVE: u8 = 0x0C;
    pub const RESPONSE: u8 = 0x0D;
    pub const RESPONSE_OK: u8 = 0x0E;
    pub const ERROR: u8 = 0x0F;

    pub mod error {
        pub const INVALID_ROOM: u8 = 0x10;
        pub const NICKNAME_COLLISION: u8 = 0x11;
        pub const SERVER_FULL: u8 = 0x12; // Not used
        pub const ALREADY_REGISTERED: u8 = 0x13;
        pub const NOT_YET_REGISTERED: u8 = 0x14;
        pub const MALFORMED: u8 = 0x15;
        pub const ALREADY_IN_ROOM: u8 = 0x16;
        pub const NOT_IN_ROOM: u8 = 0x17;
        pub const EMPTY_ROOM: u8 = 0x18;
    }
}

pub fn clear() {
    print!("\x1B[2J");
}

pub const SPACE_BYTES: &[u8] = &[0x20];

pub fn one_op_buf(opcode: u8) -> [u8; 1] {
    [opcode]
}

pub fn two_op_buf(opcode0: u8, opcode1: u8) -> [u8; 2] {
    [opcode0, opcode1]
}

pub fn one_param_buf(opcode: u8, param: &str) -> Vec<u8> {
    let opcode_buf: &[u8; 1] = &[opcode];
    let param_buf: &[u8] = param.as_bytes();
    let out_buf: Vec<u8> = [opcode_buf, param_buf].concat();
    out_buf
}

pub fn two_param_buf(opcode: u8, param0: &str, param1: &str) -> Vec<u8> {
    let opcode_buf: &[u8; 1] = &[opcode];
    let param0_buf: &[u8] = param0.as_bytes();
    let param1_buf: &[u8] = param1.as_bytes();
    let out_buf: Vec<u8> = [opcode_buf, param0_buf, SPACE_BYTES, param1_buf].concat();
    out_buf
}

pub fn three_param_buf(opcode: u8, param0: &str, param1: &str, param2: &str) -> Vec<u8> {
    let opcode_buf: &[u8; 1] = &[opcode];
    let param0_buf: &[u8] = param0.as_bytes();
    let param1_buf: &[u8] = param1.as_bytes();
    let param2_buf: &[u8] = param2.as_bytes();

    let out_buf: Vec<u8> = [
        opcode_buf,
        param0_buf,
        SPACE_BYTES,
        param1_buf,
        SPACE_BYTES,
        param2_buf,
    ]
    .concat();
    out_buf
}
