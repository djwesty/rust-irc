use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

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
        loop {
            let mut cmd_buf = [0; 2];
            println!("yo, {:?}", self.users);
            // todo!();
            break;
        }
    }
}

pub fn start() {
    let listener: TcpListener = TcpListener::bind(SERVER_ADDRESS).expect("Failed to bind to port");
    let server: Server = Server::new();
    println!("Server listening on {}", SERVER_ADDRESS);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if server.users.len() < MAX_USERS {
                    let mut server_clone = server.clone();
                    thread::spawn(move || {
                        server_clone.handle_client(stream);
                    });
                } else {
                    // stream.write_all(&[codes::ERROR, codes::error::SERVER_FULL]);
                }
            }
            Err(e) => {
                eprintln!("Error accepting connections!");
            }
        }
    }
}
