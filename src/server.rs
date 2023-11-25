use std::sync::{Arc, Mutex};
use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use prompted::input;
use rust_irc::codes;

const SERVER_ADDRESS: &str = "0.0.0.0:6667";
const MAX_USERS: usize = 20;

#[derive(Clone, Debug)]
struct Server {
    users: HashSet<String>,
    rooms: HashMap<String, Vec<String>>,
}

impl Server {
    fn new() -> Server {
        Server {
            users: HashSet::new(),
            rooms: HashMap::new(),
        }
    }

    fn handle_client(&mut self, mut stream: TcpStream) {
        // handle user commands

        loop {
            let mut buf_in: [u8; 1024] = [0; 1024];

            match stream.read(&mut buf_in) {
                Ok(size) => {
                    let cmd_bytes: &[u8] = &buf_in[0..1];
                    let param_bytes: &[u8] = &buf_in[1..size];
                    #[cfg(debug_assertions)]
                    println!("Stream in ");
                    match cmd_bytes[0] {
                        codes::client::REGISTER_NICK => {
                            #[cfg(debug_assertions)]
                            println!("REGISTER_NICK");
                            let nickname: String = String::from_utf8_lossy(param_bytes).to_string();
                            self.register_nick(nickname, &mut stream);
                        }
                        codes::client::LIST_ROOMS => {
                            #[cfg(debug_assertions)]
                            println!("LIST_ROOMS");
                            let mut buf_out: Vec<u8> = Vec::new();
                            buf_out.extend_from_slice(&[codes::RESPONSE]);
                            for (room, user) in &self.rooms {
                                buf_out.extend_from_slice(room.as_bytes());
                            }
                            stream.write(&buf_out);
                        }
                        codes::client::LIST_ROOMS => {
                            #[cfg(debug_assertions)]
                            println!("LIST_ROOMS");
                            let mut buf_out: Vec<u8> = Vec::new();
                            buf_out.extend_from_slice(&[codes::RESPONSE]);
                            for (room, user) in &self.rooms {
                                buf_out.extend_from_slice(room.as_bytes());
                            }
                            stream.write(&buf_out);
                        }
                        codes::client::LIST_USERS => {
                            #[cfg(debug_assertions)]
                            println!("LIST_USERS");
                            let mut buf_out: Vec<u8> = Vec::new();
                            buf_out.extend_from_slice(&[codes::RESPONSE]);
                            for (user) in &self.users {
                                buf_out.extend_from_slice(user.as_bytes());
                            }
                            stream.write(&buf_out);
                        }
                        codes::client::JOIN_ROOM => {
                            #[cfg(debug_assertions)]
                            println!("JOIN_ROOM");
                        }
                        codes::client::LEAVE_ROOM => {
                            #[cfg(debug_assertions)]
                            println!("LEAVE_ROOM");
                        }
                        codes::client::SEND_MESSAGE => {
                            #[cfg(debug_assertions)]
                            println!("SEND_MESSAGE");
                        }
                        _ => {
                            #[cfg(debug_assertions)]
                            println!("Unspecified client Op");
                        }
                    }
                }
                Err(_) => return,
            }
        }
    }

    fn register_nick(&mut self, nickname: String, stream: &mut TcpStream) {
        // Check for nickname collision
        if self.users.contains(&nickname) {
            #[cfg(debug_assertions)]
            println!("nickname collision, {}", nickname);
            stream.write_all(&[codes::ERROR, codes::error::NICKNAME_COLLISION]);
        } else {
            // Add the user to the user list
            self.users.insert(nickname.clone());

            // Send response ok
            stream.write_all(&[codes::RESPONSE_OK]);
        }
    }
}

pub fn start() {
    let listener: TcpListener = TcpListener::bind(SERVER_ADDRESS).expect("Failed to bind to port");
    let server = Server::new();
    let server_mutx = Arc::new(Mutex::new(server));
    let io_thread = Arc::clone(&server_mutx);

    println!("Server listening on {}", SERVER_ADDRESS);

    thread::spawn(move || loop {
        println!("0: Quit Server");
        println!("1: list connected users");
        println!("2: list rooms");
        let inp: String = input!(":");
        let local_server = io_thread.lock().unwrap();
        match inp.parse::<u8>() {
            Ok(num) => match num {
                0 => break,
                1 => println!("Users: {:?}", local_server.users),
                2 => println!("Rooms: {:?}", local_server.rooms),
                _ => println!("Invalid Input"),
            },
            Err(_) => {
                println!("Invalid input");
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut cmd_buf: [i32; 2] = [0; 2];
                let mut local_server = server_mutx.lock().unwrap();
                #[cfg(debug_assertions)]
                println!("match stream");
                if local_server.users.len() < MAX_USERS {
                    local_server.handle_client(stream);
                } else {
                    let _ = stream.write_all(&[codes::ERROR, codes::error::SERVER_FULL]);
                }
            }
            Err(e) => {
                eprintln!("Error accepting connections!");
            }
        }
    }
}
