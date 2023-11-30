use prompted::input;
use rust_irc::{clear, codes};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

use crate::client;

fn no_param_op(opcode: u8, stream: &mut TcpStream) {
    stream.write(&[opcode]).unwrap();
}

fn one_param_op(opcode: u8, stream: &mut TcpStream, param: &str) {
    let size: usize = param.to_string().capacity() + 1;
    let mut out_buf: Vec<u8> = vec![0; size];
    out_buf[0] = opcode;

    for i in 1..param.len() + 1 {
        out_buf[i] = *param.as_bytes().get(i - 1).unwrap();
    }
    stream.write(&out_buf).unwrap();
}

fn two_param_op(opcode: u8, stream: &mut TcpStream, param0: &str, param1: &str) {
    let size: usize = param0.to_string().capacity() + param1.to_string().capacity() + 2;
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

fn read_messages(mut stream: TcpStream, nick: &str) {
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Server closed the connection. Shutting down client");
                std::process::exit(0);
            }
            Ok(size) => {
                let msg_bytes: &[u8] = &buffer[..size];
                process_message(msg_bytes, nick);
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn process_message(msg_bytes: &[u8], nick: &str) {
    println!();
    match msg_bytes[0] {
        codes::ERROR => match msg_bytes[1] {
            codes::error::INVALID_ROOM => {
                println!("Operation Performed on an invalid room. Try again");
            }
            codes::error::NICKNAME_COLLISION => {
                println!("Nickname already in use on server. Connect again with a different one");
            }
            codes::error::SERVER_FULL => {
                println!("Server is full. Try again later");
            }
            _ => {
                println!("Error code: {:x?}", msg_bytes[1]);
            }
        },

        codes::client::MESSAGE => {
            let message = String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            println!("[server]:{}", message);
        }
        codes::client::MESSAGE_ROOM => {
            let params = String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            match params.split_once(" ") {
                Some((room, remainder)) => match remainder.split_once(" ") {
                    Some((user, msg)) => {
                        if user != nick {
                            println!("[{}]:[{}]: {}", room, user, msg);
                        }
                    }
                    None => {
                        eprintln!("Malformed message recieved");
                    }
                },
                None => {
                    eprintln!("Malformed message recieved");
                }
            }
        }

        codes::RESPONSE_OK => {
            // #[cfg(debug_assertions)]
            // println!("RESPONSE_OK");
        }
        codes::RESPONSE => {
            let message = String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            println!("{}", message);
        }
        codes::QUIT => {
            println!("Server has closed the connection. Stopping client");
            std::process::exit(0);
        }
        _ => {
            #[cfg(debug_assertions)]
            println!("BAD RESPONSE = {:x?} ", msg_bytes[0]);
        }
    }
}

fn disconnect(stream: &mut TcpStream) {
    stream.write_all(&[codes::QUIT]).unwrap();
    stream.shutdown(std::net::Shutdown::Both).unwrap();
}

fn help() {
    clear();
    println!("Available commands:");
    println!("/quit <- Disconnect and stop the client");
    println!("/rooms <- List all of the rooms on the server");
    println!("/users <- List all of the user connected to the server");
    println!("/list [room-name] <- List all of the users in the given room");
    println!("/join [room-name] <- Join the given room. Create the room if it does not exist");
    println!(
        "/leave [room-name] <- Leave the given room. Error if the you are not already in the room"
    );
    println!("/show [room-name] <- Switch your focus to the given room. It is suggested to join the room first");
}

pub fn start() {
    clear();
    println!("Starting the IRC client. No spaces allowed in nicknames or room names. /help to see available commands");
    let mut nick: String;
    let mut active_room: String = String::new();
    loop {
        nick = input!("Enter your nickname : ");
        if nick.contains(" ") {
            println!("May not contain spaces . Try again");
        } else if nick.is_empty() {
            println!("May not be empty . Try again");
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
        let nick_clone: String = nick.clone();
        thread::spawn(move || {
            read_messages(stream_clone, &nick_clone);
        });

        //try to register the nickname
        one_param_op(codes::client::REGISTER_NICK, &mut stream, &nick);

        loop {
            let inp: String = input!("\n[{}]:", active_room);
            let mut args: std::str::SplitWhitespace<'_> = inp.split_whitespace();
            let command: Option<&str> = args.next();

            match command {
                Some(cmd) => {
                    let param: Option<&str> = args.next();
                    match cmd {
                        "/quit" => {
                            disconnect(&mut stream);
                            break;
                        }
                        "/rooms" => no_param_op(codes::client::LIST_ROOMS, &mut stream),
                        "/users" => no_param_op(codes::client::LIST_USERS, &mut stream),
                        "/list" => match param {
                            Some(room) => {
                                one_param_op(codes::client::LIST_USERS_IN_ROOM, &mut stream, room);
                            }
                            None => {
                                println!("Room name expected, but not provided");
                            }
                        },
                        "/join" => match param {
                            Some(room) => {
                                one_param_op(codes::client::JOIN_ROOM, &mut stream, room);
                            }
                            None => {
                                println!("Room name expected, but not provided");
                            }
                        },
                        "/show" => match param {
                            Some(room) => {
                                clear();
                                active_room = room.to_string();
                            }
                            None => {
                                println!("Room name expected, but not provided")
                            }
                        },
                        "/leave" => match param {
                            Some(room) => {
                                one_param_op(codes::client::LEAVE_ROOM, &mut stream, room);
                            }
                            None => {
                                println!("Room name expected, but not provided");
                            }
                        },
                        "/msg" => match inp.split_once(" ") {
                            Some((room, msg)) => {
                                two_param_op(codes::client::MESSAGE_ROOM, &mut stream, room, msg);
                            }
                            None => {
                                println!("Usage: /msg [room] [message]");
                            }
                        },
                        "/help" => {
                            help();
                        }
                        "/" => {
                            println!("Invalid command");
                        }
                        _ => {
                            if active_room.is_empty() {
                                println!("use '/show [room]' to set an active room before sending a message");
                            } else {
                                let message: String = inp;
                                two_param_op(
                                    codes::client::MESSAGE_ROOM,
                                    &mut stream,
                                    &active_room,
                                    &message,
                                );
                            }
                        }
                    }
                }
                None => {}
            }
        }
    } else {
        println!("Failed to connect to {} with nickname {} ", host, nick);
    }
}
