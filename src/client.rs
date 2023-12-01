use prompted::input;
use rust_irc::{clear, codes};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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

fn read_messages(mut stream: TcpStream, nick: &str, timestamp: &mut Arc<Mutex<Instant>>) {
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Server closed the connection. Shutting down client");
                std::process::exit(0);
            }
            Ok(size) => {
                let mut lock = timestamp.lock().unwrap();
                *lock = Instant::now();
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
    match msg_bytes[0] {
        codes::ERROR => match msg_bytes[1] {
            codes::error::INVALID_ROOM => {
                eprintln!("Operation Performed on an invalid room. Try again");
            }
            codes::error::NICKNAME_COLLISION => {
                eprintln!("Nickname already in use on server. Connect again with a different one");
            }
            codes::error::SERVER_FULL => {
                eprintln!("Server is full. Try again later");
            }
            codes::error::NOT_IN_ROOM => {
                eprintln!("Cannot send a message before joining room. Use /join [room].")
            }
            codes::error::EMPTY_ROOM => {
                eprintln!("Room is Empty");
            }
            codes::error::ALREADY_IN_ROOM => {
                eprintln!("You are already in that room");
            }
            _ => {
                eprintln!("Error code: {:x?}", msg_bytes[1]);
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
}

pub fn start() {
    clear();
    println!("Starting the IRC client. No spaces allowed in nicknames or room names. /help to see available commands");
    let mut nick: String;
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
    let host: &str = "fab04.cecs.pdx.edu";

    if let Ok(mut stream) = TcpStream::connect(host.to_owned() + ":6667") {
        println!("Connected to {}", host);

        //another stream for reading messages
        let reader_clone: TcpStream = stream.try_clone().expect("Failed to clone stream");
        let mut keepalive_clone = stream.try_clone().expect("failed to clone stream");
        let nick_clone: String = nick.clone();

        //timestamp for detecting unresponsive server
        let timestamp = Arc::new(Mutex::new(Instant::now()));
        let mut timestamp_clone: Arc<Mutex<Instant>> = Arc::clone(&timestamp);

        thread::spawn(move || {
            read_messages(reader_clone, &nick_clone, &mut timestamp_clone);
        });

        //watchdog to send keep_alive and stop client if server fails to respond
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(5));
            let lock = timestamp.lock().unwrap();
            let now = Instant::now();
            if now.duration_since(*lock) > Duration::from_secs(30) {
                eprintln!("Server is unresponsive. Stopping client");
                std::process::exit(1);
            } else if now.duration_since(*lock) > Duration::from_secs(5) {
                keepalive_clone.write_all(&[codes::KEEP_ALIVE]).unwrap();
            }
        });

        //try to register the nickname
        one_param_op(codes::client::REGISTER_NICK, &mut stream, &nick);

        loop {
            let inp = input!("");

            match inp.split_once(" ") {
                Some((cmd, param)) => match cmd {
                    "/quit" => {
                        disconnect(&mut stream);
                        break;
                    }
                    "/rooms" => no_param_op(codes::client::LIST_ROOMS, &mut stream),
                    "/users" => no_param_op(codes::client::LIST_USERS, &mut stream),
                    "/list" => match param.split_once(" ") {
                        Some((room, _)) => {
                            one_param_op(codes::client::LIST_USERS_IN_ROOM, &mut stream, room);
                        }
                        _ => {
                            println!("Malformaed. Try /list [room-name]");
                        }
                    },
                    "/join" => match param.split_once(" ") {
                        Some((_, _)) => {
                            println!("Malformed. Try /join [room-name]");
                        }
                        _ => {
                            one_param_op(codes::client::JOIN_ROOM, &mut stream, param);
                        }
                    },

                    "/leave" => match param.split_once(" ") {
                        Some((_, _)) => {
                            println!("Malformed. Try /leave [room-name]");
                        }
                        _ => {
                            one_param_op(codes::client::LEAVE_ROOM, &mut stream, param);
                        }
                    },
                    "/msg" => match param.split_once(" ") {
                        Some((room, msg)) => {
                            two_param_op(codes::client::MESSAGE_ROOM, &mut stream, room, msg);
                        }
                        _ => {
                            println!("Ufsage: /msg [room] [message]");
                        }
                    },
                    "/help" => {
                        help();
                    }
                    "/" => {
                        println!("Invalid command");
                    }
                    _ => {
                        one_param_op(codes::client::MESSAGE, &mut stream, &inp);
                    }
                },
                _ => {
                    one_param_op(codes::client::MESSAGE, &mut stream, &inp);
                }
            }
        }
    } else {
        println!("Failed to connect to {} with nickname {} ", host, nick);
    }
}
