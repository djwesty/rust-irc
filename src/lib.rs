pub mod codes {
    pub const TBD: u8 = 0x00;
    pub mod client {
        pub const JOIN_ROOM: u8 = 0x01;
        pub const JOIN_SERVER: u8 = 0x02;
        pub const LEAVE_ROOM: u8 = 0x03;
        pub const LIST_ROOMS: u8 = 0x04;
        pub const MESSAGE: u8 = 0x05;
        pub const REGISTER_NICK: u8 = 0x06;
        pub const LIST_USERS: u8 = 0x07;
        pub const LIST_USERS_IN_ROOM: u8 = 0x08;
        pub const MESSAGE_ROOM: u8 = 0x09;
    }
    pub const QUIT: u8 = 0x0B;
    pub const KEEP_ALIVE: u8 = 0x0C;
    pub const RESPONSE: u8 = 0x0D;
    pub const RESPONSE_OK: u8 = 0x0E;
    pub const ERROR: u8 = 0x0F;

    pub mod error {
        pub const INVALID_ROOM: u8 = 0x10;
        pub const NICKNAME_COLLISION: u8 = 0x11;
        pub const SERVER_FULL: u8 = 0x12;
        pub const ALREADY_REGISTERED: u8 = 0x13;
        pub const NOT_YET_REGISTERED: u8 = 0x14;
        pub const MALFORMED: u8 = 0x15;
        pub const ALREADY_IN_ROOM: u8 = 0x16;
        pub const NOT_IN_ROOM: u8 = 0x17;
    }
}

pub fn clear() {
    print!("\x1B[2J");
}
