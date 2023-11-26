use prompted::input;
use rust_irc::codes;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

fn no_param_op(opcode: u8, stream: &mut TcpStream) {
    stream.write(&[opcode]).unwrap();
}

fn one_param_op(opcode: u8, stream: &mut TcpStream, param: &str) {
    let size = param.to_string().capacity() + 1;
    let mut out_buf: Vec<u8> = vec![0; size];
    out_buf[0] = opcode;

    for i in 1..param.len() + 1 {
        out_buf[i] = *param.as_bytes().get(i - 1).unwrap();
    }
    stream.write(&out_buf).unwrap();
}

fn two_param_op(opcode: u8, stream: &mut TcpStream, param0: &str, param1: &str) {
    let size = param0.to_string().capacity() + param1.to_string().capacity() + 2;
    let mut out_buf: Vec<u8> = vec![0; size];
    let mut byte: usize = 0;
    out_buf[byte] = opcode;
    byte += 1;
    for i in 0..param0.len() {
        out_buf[byte] = *param0.as_bytes().get(i).unwrap();
        byte += 1;
    }
    out_buf[byte] = 0x20;
    byte += 1;
    for i in 0..param1.len() {
        out_buf[byte] = *param1.as_bytes().get(i).unwrap();
        byte += 1;
    }
    stream.write(&out_buf).unwrap();
}

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

fn show(stream: &mut TcpStream) {}

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
        stream.write(&buf).unwrap();

        loop {
            let inp: String = input!(":");
            let cmds: Vec<_> = inp.split(" ").collect();
            match *cmds.get(0).unwrap() {
                "/quit" => disconnect(),
                "/rooms" => no_param_op(codes::client::LIST_ROOMS, &mut stream),
                "/users" => no_param_op(codes::client::LIST_USERS, &mut stream),
                "/list" => {
                    let room = *cmds.get(1).unwrap();
                    one_param_op(codes::client::LIST_USERS_IN_ROOM, &mut stream, room);
                }
                "/join" => {
                    let room = *cmds.get(1).unwrap();
                    one_param_op(codes::client::JOIN_ROOM, &mut stream, room);
                }
                "/show" => show(&mut stream),
                "/leave" => {
                    let room: &str = *cmds.get(1).unwrap();
                    one_param_op(codes::client::LEAVE_ROOM, &mut stream, room);
                }
                "/msg" => {
                    let room: &str = *cmds.get(1).unwrap();
                    // let message = *cmds.
                    // two_param_op(
                    //     codes::client::SEND_MESSAGE_TO_ROOM,
                    //     &mut stream,
                    //     param0,
                    //     param1,
                    // )
                }

                _ => {
                    println!("Not implemented");
                }
            }
        }
    } else {
        println!("Failed to connect to {} with nickname {} ", host, nick);
    }
}
