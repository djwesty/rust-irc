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
        let mut buffer: [u8; 1024] = [0; 1024];
        let nickname: String;

        // Read the nickname from the client
        match stream.read(&mut buffer) {
            Ok(size) => {
                let nickname_bytes = &buffer[0..size];
                nickname = String::from_utf8_lossy(nickname_bytes).to_string();
            }
            Err(_) => return,
        }

        // Check for nickname collision
        if self.users.contains(&nickname) {
            stream.write_all(&[codes::ERROR, codes::error::NICKNAME_COLLISION]);
            return;
        }

        // Add the user to the user list
        self.users.insert(nickname.clone());

        // Send response ok
        stream.write_all(&[codes::RESPONSE_OK]);

        // handle user commands
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
