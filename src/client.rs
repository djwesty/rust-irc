use prompted::input;
use rust_irc::codes;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

fn read_messages(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    break; //Server closed connection
                }
                let msg_bytes: &[u8] = &buffer[..size];
                process_message(msg_bytes);
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn process_message(msg_bytes: &[u8]) {
    match msg_bytes[0] {
        codes::ERROR => {
            #[cfg(debug_assertions)]
            println!("err: {:x?}", msg_bytes[1]);
            match msg_bytes[1] {
                codes::error::INVALID_ROOM => {
                    println!("Attempted to message or list non-existant room. Try again");
                }
                codes::error::NICKNAME_COLLISION => {
                    println!(
                        "Nickname already in use on server. Connect again with a different one"
                    );
                    disconnect();
                }
                codes::error::SERVER_FULL => {
                    println!("Server is full. Try again later");
                    disconnect();
                }
                _ => {
                    #[cfg(debug_assertions)]
                    println!("Unknown error code {:x?}", msg_bytes[1]);
                }
            }
        }
        codes::RESPONSE_OK => {
            #[cfg(debug_assertions)]
            println!("RESPONSE_OK");
        }
        codes::RESPONSE => {
            let message = String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            println!("{}", message);
        }
        _ => {
            #[cfg(debug_assertions)]
            println!("BAD RESPONSE = {:x?} ", msg_bytes[0]);
        }
    }
}

fn disconnect() {}

fn rooms(stream: &mut TcpStream) {
    stream.write(&[codes::client::LIST_ROOMS]);
}
fn users(stream: &mut TcpStream) {
    stream.write(&[codes::client::LIST_USERS]);
}

fn msg(stream: &mut TcpStream) {}

fn join(nick: &str, room: &str, stream: &mut TcpStream) {
    let size = room.to_string().capacity() + nick.to_string().capacity() + 2;
    let mut out_buf: Vec<u8> = vec![0; size];
    let mut byte: usize = 0;
    out_buf[byte] = codes::client::JOIN_ROOM;
    byte += 1;
    for i in 0..nick.len() {
        out_buf[byte] = *nick.as_bytes().get(i).unwrap();
        byte += 1;
    }
    out_buf[byte] = 0x20;
    byte += 1;
    for i in 0..room.len() {
        out_buf[byte] = *room.as_bytes().get(i).unwrap();
        byte += 1;
    }
    stream.write(&out_buf);
}

fn show(stream: &mut TcpStream) {}

fn leave(nick: &str, room: &str, stream: &mut TcpStream) {
    let size = room.to_string().capacity() + nick.to_string().capacity() + 2;
    let mut out_buf: Vec<u8> = vec![0; size];
    let mut byte: usize = 0;
    out_buf[byte] = codes::client::LEAVE_ROOM;
    byte += 1;
    for i in 0..nick.len() {
        out_buf[byte] = *nick.as_bytes().get(i).unwrap();
        byte += 1;
    }
    out_buf[byte] = 0x20;
    byte += 1;
    for i in 0..room.len() {
        out_buf[byte] = *room.as_bytes().get(i).unwrap();
        byte += 1;
    }
    stream.write(&out_buf);
}

fn list( room: &str, stream: &mut TcpStream) {
    let size = room.to_string().capacity() +1;
    let mut out_buf: Vec<u8> = vec![0; size];
    out_buf[0] = codes::client::LIST_USERS_IN_ROOM;


    for i in 1..room.len()+1 {
        out_buf[i] = *room.as_bytes().get(i-1).unwrap();
    }
    stream.write(&out_buf);
}

pub fn start() {
    println!("Starting the IRC client. No spaces allowed in nicknames or room names");
    let mut nick: String;
    loop {
        nick = input!("Enter your nickname : ");
        if nick.contains(" ") {
            println!("May not contain spaces. Try again");
        } else {
            break;
        }
    }

    // let host: String = input!("Enter the server host: ");
    let host: &str = "localhost";

    if let Ok(mut stream) = TcpStream::connect(host.to_owned() + ":6667") {
        println!("Connected to {}", host);

        //another stream for reading messages
        let stream_clone: TcpStream = stream.try_clone().expect("Failed to clone stream");
        thread::spawn(move || {
            read_messages(stream_clone);
        });

        //try to register the nickname
        let mut buf: Vec<u8> = vec![0; nick.capacity()];
        buf[0] = codes::client::REGISTER_NICK;
        for i in 1..nick.len() + 1 {
            buf[i] = *nick.as_bytes().get(i - 1).unwrap();
        }
        stream.write(&buf);

        loop {
            let inp: String = input!(":");
            let cmds: Vec<_> = inp.split(" ").collect();
            match *cmds.get(0).unwrap() {
                "/quit" => disconnect(),
                "/rooms" => rooms(&mut stream),
                "/users" => users(&mut stream),
                "/list" => {
                    let room = *cmds.get(1).unwrap();
                    list(room, &mut stream);

                }
                "/join" => {
                    let room = *cmds.get(1).unwrap();
                    join(&nick, room, &mut stream)
                }
                "/show" => show(&mut stream),
                "/leave" => {
                    let room = *cmds.get(1).unwrap();
                    leave(&nick, room, &mut stream)
                }
                "/msg" => msg(&mut stream),

                _ => msg(&mut stream),
            }
        }
    } else {
        println!("Failed to connect to {} with nickname {} ", host, nick);
    }
}
