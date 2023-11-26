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

#[derive(Debug)]
struct Server {
    users: HashSet<String>,
    rooms: HashMap<String, Vec<String>>,
}

impl Server {
    fn new() -> Self {
        Server {
            users: HashSet::new(),
            rooms: HashMap::new(),
        }
    }
}

fn handle_client(
    server: &Arc<Mutex<Server>>,
    stream: &mut TcpStream,
    cmd_bytes: &[u8],
    param_bytes: &[u8],
) {
    // handle user commands
    match cmd_bytes[0] {
        codes::client::REGISTER_NICK => {
            let nickname: String = String::from_utf8_lossy(param_bytes).to_string();
            register_nick(server, nickname, stream);
        }
        codes::client::LIST_ROOMS => {
            let unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
            let mut buf_out: Vec<u8> = Vec::new();
            buf_out.extend_from_slice(&[codes::RESPONSE]);
            for (room, user) in &unlocked_server.rooms {
                buf_out.extend_from_slice(room.as_bytes());
                buf_out.extend_from_slice(&[0x20]);
            }
            stream.write(&buf_out);
        }

        codes::client::LIST_USERS => {
            let unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
            let mut buf_out: Vec<u8> = Vec::new();
            buf_out.extend_from_slice(&[codes::RESPONSE]);
            for (user) in &unlocked_server.users {
                buf_out.extend_from_slice(user.as_bytes());
                buf_out.extend_from_slice(&[0x20]);
            }
            stream.write(&buf_out);
        }

        codes::client::JOIN_ROOM => {
            let p: String = String::from_utf8_lossy(param_bytes).to_string();
            let params: Vec<&str> = p.split(' ').collect();
            let user = params.get(0).unwrap();
            let room = params.get(1).unwrap();
            join_room(server, *user, room, stream)
        }

        codes::client::LEAVE_ROOM => {
            let p: String = String::from_utf8_lossy(param_bytes).to_string();
            let params: Vec<&str> = p.split(' ').collect();
            let user = params.get(0).unwrap();
            let room = params.get(1).unwrap();
            leave_room(server, user, room, stream)
        }
        
        codes::client::SEND_MESSAGE => {
            #[cfg(debug_assertions)]
            println!("SEND_MESSAGE");
        }
        _ => {
            #[cfg(debug_assertions)]
            println!("Unspecified client Op, {:x?}", cmd_bytes);
        }
    }

    // }
}

fn register_nick(server: &Arc<Mutex<Server>>, nickname: String, stream: &mut TcpStream) {
    // Check for nickname collision
    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    if unlocked_server.users.contains(&nickname) {
        #[cfg(debug_assertions)]
        println!("Nickname Collision, {}", nickname);
        stream.write_all(&[codes::ERROR, codes::error::NICKNAME_COLLISION]);
    } else {
        // Add the user to the user list
        unlocked_server.users.insert(nickname.clone());

        // Send response ok
        stream.write_all(&[codes::RESPONSE_OK]);
    }
}

fn join_room(server: &Arc<Mutex<Server>>, user: &str, room: &str, stream: &mut TcpStream) {
    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();

    match unlocked_server.rooms.get_mut(room) {
        Some(l) => {
            l.push(user.to_string());
        }
        None => {
            let list: Vec<String> = vec![user.to_string()];
            unlocked_server.rooms.insert(room.to_string(), list);
        }
    }
    stream.write_all(&[codes::RESPONSE_OK]);
}

fn leave_room(server: &Arc<Mutex<Server>>, user: &str, room: &str, stream: &mut TcpStream) {
    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    match unlocked_server.rooms.get_mut(room) {
        Some(l) => {
            let before_len = l.len();
            l.retain(|item| item != user);
            if l.len() == 0 {
                unlocked_server.rooms.remove(room);
            } else if l.len() == before_len {
                stream.write_all(&[codes::ERROR, codes::error::INVALID_ROOM]);
            } else {
                stream.write_all(&[codes::RESPONSE_OK]);
            }
        }
        None => {
            stream.write_all(&[codes::ERROR, codes::error::INVALID_ROOM]);
        }
    }
}

pub fn start() {
    let listener: TcpListener = TcpListener::bind(SERVER_ADDRESS).expect("Failed to bind to port");
    let server: Arc<Mutex<Server>> = Arc::new(Mutex::new(Server::new()));
    let server_outer: Arc<Mutex<Server>> = Arc::clone(&server);
    println!("Server listening on {}", SERVER_ADDRESS);

    thread::spawn(move || {
        for tcpstream in listener.incoming() {
            match tcpstream {
                Ok(mut stream) => {
                    let mut buf_in: [u8; 1024] = [0; 1024];
                    let server_inner: Arc<Mutex<Server>> = Arc::clone(&server_outer);

                    thread::spawn(move || loop {
                        match stream.read(&mut buf_in) {
                            Ok(size) => {
                                let cmd_bytes: &[u8] = &buf_in[0..1];
                                let param_bytes: &[u8] = &buf_in[1..size];

                                handle_client(&server_inner, &mut stream, cmd_bytes, param_bytes);
                            }
                            Err(_) => {
                                eprintln!("Error parsing client");
                                stream.write(&[codes::END]);
                                break;
                            }
                        }
                    });
                }
                Err(_) => {
                    eprintln!("Error accepting connections!");
                }
            }
        }
    });

    loop {
        println!("0: Quit Server");
        println!("1: list connected users");
        println!("2: list rooms");
        let inp: String = input!(":");
        match inp.parse::<u8>() {
            Ok(num) => match num {
                0 => break,
                1 => println!("Users: {:?}", server.lock().unwrap().users),
                2 => println!("Rooms: {:?}", server.lock().unwrap().rooms),
                _ => println!("Invalid Input"),
            },
            Err(_) => {
                println!("Invalid input");
            }
        }
    }
}
