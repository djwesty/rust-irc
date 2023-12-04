use prompted::input;
use rust_irc::{clear, codes, one_op_buf, one_param_buf, two_param_buf, DEFAULT_PORT};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_HOST: &str = "localhost";

fn read_messages(mut stream: TcpStream, nick: &str, timestamp: &mut Arc<Mutex<Instant>>) {
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Server closed the connection. Shutting down client");
                std::process::exit(0);
            }
            Ok(size) => {
                let mut lock: std::sync::MutexGuard<'_, Instant> = timestamp.lock().unwrap();
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
                std::process::exit(1);
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

        codes::MESSAGE => {
            let message: String =
                String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            println!("[server]:{}", message);
        }
        codes::MESSAGE_ROOM => {
            let params: String = String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            match params.split_once(' ') {
                Some((room, remainder)) => match remainder.split_once(' ') {
                    Some((user, msg)) => {
                        if user != nick {
                            println!("[{}]:[{}]: {}", room, user, msg);
                        }
                    }
                    None => {
                        eprintln!("Malformed message recieved: {}", params);
                    }
                },
                None => {
                    eprintln!("Malformed message recieved: {}", params);
                }
            }
        }

        codes::RESPONSE_OK => {}
        codes::RESPONSE => {
            let message = String::from_utf8(msg_bytes[1..msg_bytes.len()].to_vec()).unwrap();
            println!("{}", message);
        }
        codes::QUIT => {
            println!("Server has closed the connection. Stopping client");
            std::process::exit(0);
        }
        _ => {
            eprintln!("BAD RESPONSE = {:x?} ", msg_bytes[0]);
        }
    }
}

fn disconnect(stream: &mut TcpStream) {
    stream.write_all(&[codes::QUIT]).unwrap();
    stream.shutdown(std::net::Shutdown::Both).unwrap();
}

fn help() {
    println!("Available commands:");
    println!("/quit <- Disconnect and stop the client");
    println!("/rooms <- List all of the rooms on the server");
    println!("/users <- List all of the user connected to the server");
    println!("/list [room-name] <- List all of the users in the given room");
    println!("/join [room-name] <- Join the given room. Create the room if it does not exist");
    println!(
        "/leave [room-name] <- Leave the given room. Error if you are not already in the room"
    );
}

pub fn start() {
    clear();
    println!("Starting the IRC client. No spaces allowed in nicknames or room names.");
    let mut nick: String;
    loop {
        nick = input!("Enter your nickname : ");
        if nick.contains(' ') {
            println!("May not contain spaces . Try again");
        } else if nick.is_empty() {
            println!("May not be empty . Try again");
        } else {
            break;
        }
    }

    let mut host: String = input!("Enter the server host (empty for {}): ", DEFAULT_HOST);
    if host.is_empty() {
        host = DEFAULT_HOST.to_owned();
    }
    host.push(':');
    host.push_str(&DEFAULT_PORT.to_string());
    if let Ok(mut stream) = TcpStream::connect(host.to_owned()) {
        println!("Connected to {}.\n/help to see available commands", host);

        //another stream for reading messages
        let reader_clone: TcpStream = stream.try_clone().expect("Failed to clone stream");
        let mut keepalive_clone = stream.try_clone().expect("failed to clone stream");
        let nick_clone: String = nick.clone();

        //timestamp for detecting unresponsive server
        let timestamp: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
        let mut timestamp_clone: Arc<Mutex<Instant>> = Arc::clone(&timestamp);

        thread::spawn(move || {
            read_messages(reader_clone, &nick_clone, &mut timestamp_clone);
        });

        //watchdog to send keep_alive and stop client if server fails to respond
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(5));
            let lock: std::sync::MutexGuard<'_, Instant> = timestamp.lock().unwrap();
            let now: Instant = Instant::now();
            if now.duration_since(*lock) > Duration::from_secs(30) {
                eprintln!("Server is unresponsive. Stopping client");
                std::process::exit(1);
            } else if now.duration_since(*lock) > Duration::from_secs(5) {
                keepalive_clone.write_all(&[codes::KEEP_ALIVE]).unwrap();
            }
        });

        //try to register the nickname
        let nick_reg_buff: Vec<u8> = one_param_buf(codes::REGISTER_NICK, &nick);
        stream.write_all(&nick_reg_buff).unwrap();
        loop {
            let inp: String = input!("");

            match inp.split_once(' ') {
                Some((cmd, param)) => match cmd {
                    "/list" => match param.split_once(' ') {
                        Some((_, _)) => {
                            eprintln!("Malformaed. Try /list [room-name]");
                        }
                        _ => {
                            let out_buf: Vec<u8> = one_param_buf(codes::LIST_USERS_IN_ROOM, param);
                            stream.write_all(&out_buf).unwrap();
                        }
                    },
                    "/join" => match param.split_once(' ') {
                        Some((_, _)) => {
                            eprintln!("Malformed. Try /join [room-name]");
                        }
                        _ => {
                            let out_buf: Vec<u8> = one_param_buf(codes::JOIN_ROOM, param);
                            stream.write_all(&out_buf).unwrap();
                        }
                    },

                    "/leave" => match param.split_once(' ') {
                        Some((_, _)) => {
                            eprintln!("Malformed. Try /leave [room-name]");
                        }
                        _ => {
                            let out_buf: Vec<u8> = one_param_buf(codes::LEAVE_ROOM, param);
                            stream.write_all(&out_buf).unwrap();
                        }
                    },
                    "/msg" => match param.split_once(' ') {
                        Some((room, msg)) => {
                            let out_buf = two_param_buf(codes::MESSAGE_ROOM, room, msg);
                            stream.write_all(&out_buf).unwrap();
                        }
                        _ => {
                            eprintln!("Usage: /msg [room] [message]");
                        }
                    },
                    _ => {
                        let out_buf: Vec<u8> = one_param_buf(codes::MESSAGE, &inp);
                        stream.write_all(&out_buf).unwrap();
                    }
                },

                _ => match inp.as_str() {
                    "/quit" => {
                        disconnect(&mut stream);
                        break;
                    }
                    "/rooms" => {
                        let out_buf: [u8; 1] = one_op_buf(codes::LIST_ROOMS);
                        stream.write_all(&out_buf).unwrap();
                    }
                    "/users" => {
                        let out_buf: [u8; 1] = one_op_buf(codes::LIST_USERS);
                        stream.write_all(&out_buf).unwrap();
                    }
                    "/help" => {
                        help();
                    }
                    "/" => {
                        eprintln!("Invalid command");
                    }
                    _ => {
                        let out_buf: Vec<u8> = one_param_buf(codes::MESSAGE, &inp);
                        stream.write_all(&out_buf).unwrap();
                    }
                },
            }
        }
    } else {
        println!("Failed to connect to {} with nickname {} ", host, nick);
    }
}
