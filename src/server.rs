use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use prompted::input;
use rust_irc::{clear, codes};

const SERVER_ADDRESS: &str = "0.0.0.0:6667";
const MAX_USERS: usize = 20;

#[derive(Debug)]
struct Server {
    users: HashMap<String, TcpStream>,
    rooms: HashMap<String, Vec<String>>,
}

impl Server {
    fn new() -> Self {
        Server {
            users: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

fn message(room: &str, msg: &str, sender: &str, server: &Arc<Mutex<Server>>) {
    let size = room.len() + msg.len() + sender.len() + 3;
    let mut out_buf: Vec<u8> = vec![0; size];

    let mut byte: usize = 0;
    out_buf[byte] = codes::client::MESSAGE_ROOM;
    byte += 1;

    for i in 0..room.len() {
        out_buf[byte] = *room.as_bytes().get(i).unwrap();
        byte += 1;
    }

    out_buf[byte] = 0x20;
    byte += 1;

    for i in 0..sender.len() {
        out_buf[byte] = *sender.as_bytes().get(i).unwrap();
        byte += 1;
    }

    out_buf[byte] = 0x20;
    byte += 1;

    for i in 0..msg.len() {
        out_buf[byte] = *msg.as_bytes().get(i).unwrap();
        byte += 1;
    }

    let mut guard: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    let server: &mut Server = guard.deref_mut();

    //1: Make sure specified rooms exists, ifn error
    //2: Make sure sender is a member of the room, ifn error
    //3: Message all non-sender users in the room the message, ifnone error empty room
    //4: Message the sender RESPONSE_OK
    let room_users: Option<&Vec<String>> = server.rooms.get(room);
    let mut sender_stream = server
        .users
        .get_mut(sender)
        .unwrap()
        .try_clone()
        .expect("Clone issue");
    //1
    match room_users {
        Some(users) => {
            //2
            let mut is_member = false;
            for user in users {
                if user.eq(sender) {
                    is_member = true;
                }
            }

            if is_member {
                for user in users {
                    if user.eq(sender) {
                        //4
                        sender_stream.write_all(&[codes::RESPONSE_OK]).unwrap();
                    } else {
                        //3
                        let recipient_stream: Option<&mut TcpStream> = server.users.get_mut(user);
                        match recipient_stream {
                            Some(str) => {
                                println!("Sending msg {:?}", out_buf.to_ascii_lowercase());
                                str.write_all(&out_buf).unwrap();
                            }
                            None => {
                                eprintln!("Server error: could not find user");
                            }
                        }
                    }
                }
            } else {
                sender_stream
                    .write_all(&[codes::ERROR, codes::error::NOT_IN_ROOM])
                    .unwrap();
            }
        }
        None => {
            sender_stream
                .write_all(&[codes::ERROR, codes::error::EMPTY_ROOM])
                .unwrap();
        }
    }
}

fn broadcast(op: u8, server: &Arc<Mutex<Server>>, message: &str) {
    let size: usize = message.len() + 1;
    let mut out_buf: Vec<u8> = vec![0; size];
    out_buf[0] = op;

    for i in 1..size {
        out_buf[i] = *message.as_bytes().get(i - 1).unwrap();
    }

    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    let streams: std::collections::hash_map::ValuesMut<'_, String, TcpStream> =
        unlocked_server.users.values_mut();
    for stream in streams {
        stream.write_all(&out_buf).unwrap();
    }
}

fn disconnect_all(server: &Arc<Mutex<Server>>) {
    let mut guard: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    let users: std::collections::hash_map::ValuesMut<'_, String, TcpStream> =
        guard.users.values_mut();
    users.for_each(|user: &mut TcpStream| {
        user.write(&[codes::QUIT]).unwrap();
        user.shutdown(std::net::Shutdown::Both).unwrap();
    })
}

fn handle_client(
    server: &Arc<Mutex<Server>>,
    stream: &mut TcpStream,
    nickname: &str,
    cmd_bytes: &[u8],
    param_bytes: &[u8],
) {
    // handle user commands
    match cmd_bytes[0] {
        codes::client::REGISTER_NICK => {
            stream
                .write_all(&[codes::ERROR, codes::error::ALREADY_REGISTERED])
                .unwrap();
        }
        codes::client::LIST_ROOMS => {
            let unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
            let mut buf_out: Vec<u8> = Vec::new();
            buf_out.extend_from_slice(&[codes::RESPONSE]);
            for (room, _user) in &unlocked_server.rooms {
                buf_out.extend_from_slice(room.as_bytes());
                buf_out.extend_from_slice(&[0x20]);
            }
            stream.write(&buf_out).unwrap();
        }

        codes::client::LIST_USERS => {
            let unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
            let mut buf_out: Vec<u8> = Vec::new();
            buf_out.extend_from_slice(&[codes::RESPONSE]);
            for (user, _) in &unlocked_server.users {
                buf_out.extend_from_slice(user.as_bytes());
                buf_out.extend_from_slice(&[0x20]);
            }
            stream.write(&buf_out).unwrap();
        }

        codes::client::LIST_USERS_IN_ROOM => {
            let room: String = String::from_utf8_lossy(param_bytes).to_string();
            let unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
            let mut buf_out: Vec<u8> = Vec::new();
            buf_out.extend_from_slice(&[codes::RESPONSE]);
            match unlocked_server.rooms.get(&room) {
                Some(l) => {
                    for ele in l {
                        buf_out.extend_from_slice(ele.as_bytes());
                        buf_out.extend_from_slice(&[0x20]);
                    }
                    stream.write_all(&buf_out).unwrap();
                }
                None => {
                    stream
                        .write_all(&[codes::ERROR, codes::error::INVALID_ROOM])
                        .unwrap();
                }
            }
        }

        codes::client::JOIN_ROOM => {
            let p: String = String::from_utf8_lossy(param_bytes).to_string();
            let params: Vec<&str> = p.split_whitespace().collect();
            let room = params.get(0).unwrap();
            join_room(server, &nickname, room, stream);
        }

        codes::client::LEAVE_ROOM => {
            let p: String = String::from_utf8_lossy(param_bytes).to_string();
            let params: Vec<&str> = p.split_whitespace().collect();
            let room = params.get(0).unwrap();
            leave_room(server, &nickname, room, stream);
        }

        codes::client::MESSAGE => {
            #[cfg(debug_assertions)]
            println!("MESSAGE");
            stream.write_all(&[codes::RESPONSE_OK]).unwrap();
        }

        codes::KEEP_ALIVE => {
            println!("kEEP alive");
            stream.write_all(&[codes::RESPONSE_OK]).unwrap();
        }

        codes::client::MESSAGE_ROOM => {
            let p: String = String::from_utf8_lossy(param_bytes).to_string();
            let params: Option<(&str, &str)> = p.split_once(" ");

            match params {
                Some((room, msg)) => {
                    message(room, msg, nickname, server);
                }
                _ => {
                    stream
                        .write(&[codes::ERROR, codes::error::MALFORMED])
                        .unwrap();
                }
            }
        }
        codes::QUIT => {
            remove_user(server, nickname, stream);
        }
        _ => {
            #[cfg(debug_assertions)]
            println!("Unspecified client Op, {:x?}", cmd_bytes);
        }
    }

    // }
}

fn remove_user(server: &Arc<Mutex<Server>>, nickname: &str, stream: &mut TcpStream) {
    let mut guard: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    let server: &mut Server = guard.deref_mut();
    let mut rooms: &mut HashMap<String, Vec<String>> = &mut server.rooms;
    rooms.values_mut().for_each(|room: &mut Vec<String>| {
        room.retain(|u: &String| !u.eq(nickname));
    });
    let users: &mut HashMap<String, TcpStream> = &mut server.users;
    users.remove(nickname);
}

fn register_nick(server: &Arc<Mutex<Server>>, nickname: &str, stream: &mut TcpStream) {
    // Check for nickname collision
    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    if unlocked_server.users.contains_key(nickname) {
        #[cfg(debug_assertions)]
        println!("Nickname Collision, {}", nickname);
        stream
            .write_all(&[codes::ERROR, codes::error::NICKNAME_COLLISION])
            .unwrap();
    } else {
        // Add the user to the user list
        let clone: TcpStream = stream.try_clone().expect("fail to clone");
        let addr: String = clone.peer_addr().unwrap().to_string();

        unlocked_server.users.insert(nickname.to_string(), clone);
        // Send response ok
        stream.write_all(&[codes::RESPONSE_OK]).unwrap();

        println!("{} has registered nickname {}", addr, nickname);
    }
}

fn join_room(server: &Arc<Mutex<Server>>, user: &str, room: &str, stream: &mut TcpStream) {
    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();

    match unlocked_server.rooms.get_mut(room) {
        Some(l) => {
            for ele in l.into_iter() {
                if ele == user {
                    stream
                        .write_all(&[codes::ERROR, codes::error::ALREADY_IN_ROOM])
                        .unwrap();
                    return;
                }
            }
            l.push(user.to_string());
        }
        None => {
            let list: Vec<String> = vec![user.to_string()];
            unlocked_server.rooms.insert(room.to_string(), list);
        }
    }
    stream.write_all(&[codes::RESPONSE_OK]).unwrap();
}

fn leave_room(server: &Arc<Mutex<Server>>, user: &str, room: &str, stream: &mut TcpStream) {
    let mut unlocked_server: std::sync::MutexGuard<'_, Server> = server.lock().unwrap();
    match unlocked_server.rooms.get_mut(room) {
        Some(l) => {
            let before_len: usize = l.len();
            l.retain(|item: &String| item != user);
            if l.len() == 0 {
                unlocked_server.rooms.remove(room);
                stream.write_all(&[codes::RESPONSE_OK]).unwrap();
            } else if l.len() == before_len {
                stream
                    .write_all(&[codes::ERROR, codes::error::INVALID_ROOM])
                    .unwrap();
            } else {
                stream.write_all(&[codes::RESPONSE_OK]).unwrap();
            }
        }
        None => {
            stream
                .write_all(&[codes::ERROR, codes::error::INVALID_ROOM])
                .unwrap();
        }
    }
}

pub fn start() {
    let listener: TcpListener = TcpListener::bind(SERVER_ADDRESS).expect("Failed to bind to port");
    let server: Arc<Mutex<Server>> = Arc::new(Mutex::new(Server::new()));
    let server_outer: Arc<Mutex<Server>> = Arc::clone(&server);
    clear();
    println!("Server listening on {}", SERVER_ADDRESS);

    thread::spawn(move || {
        for tcpstream in listener.incoming() {
            match tcpstream {
                Ok(mut stream) => {
                    let mut buf_in: [u8; 1024] = [0; 1024];
                    let server_inner: Arc<Mutex<Server>> = Arc::clone(&server_outer);

                    thread::spawn(move || {
                        let nickname: String;
                        println!(
                            "IP {} has connected",
                            stream.peer_addr().unwrap().to_string()
                        );
                        match stream.read(&mut buf_in) {
                            Ok(0) => {
                                println!(
                                    "IP {} has closed the connection",
                                    stream.peer_addr().unwrap().to_string()
                                );
                            }
                            Ok(size) => {
                                let cmd_bytes: &[u8] = &buf_in[0..1];
                                let param_bytes: &[u8] = &buf_in[1..size];
                                if cmd_bytes[0] == codes::client::REGISTER_NICK {
                                    nickname = String::from_utf8_lossy(param_bytes).to_string();
                                    register_nick(&server_inner, &nickname, &mut stream);
                                    loop {
                                        match stream.read(&mut buf_in) {
                                            Ok(0) => {
                                                println!("IP {} with nickname {} has closed the connection", stream.peer_addr().unwrap().to_string(), nickname);
                                                remove_user(&server_inner, &nickname, &mut stream);
                                                break;
                                            }
                                            Ok(size) => {
                                                let cmd_bytes: &[u8] = &buf_in[0..1];
                                                let param_bytes: &[u8] = &buf_in[1..size];

                                                handle_client(
                                                    &server_inner,
                                                    &mut stream,
                                                    &nickname,
                                                    cmd_bytes,
                                                    param_bytes,
                                                );
                                            }
                                            Err(_) => {
                                                eprintln!("Error parsing client");
                                                stream.write(&[codes::QUIT]).unwrap();
                                                break;
                                            }
                                        }
                                    }
                                } else {
                                    stream
                                        .write_all(&[
                                            codes::ERROR,
                                            codes::error::NOT_YET_REGISTERED,
                                        ])
                                        .unwrap();
                                }
                            }
                            Err(_) => {
                                eprintln!("Error parsing client");
                                stream.write(&[codes::QUIT]).unwrap();
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
        println!("3: Broadcast message to all");
        println!("4: Freeze server via double lock (for testing)");
        let inp: String = input!(":");
        match inp.parse::<u8>() {
            Ok(num) => match num {
                0 => {
                    println!("Stopping Server");
                    disconnect_all(&server);
                    break;
                }
                1 => println!("Users: {:?}", server.lock().unwrap().users),
                2 => println!("Rooms: {:?}", server.lock().unwrap().rooms),
                3 => {
                    let inp2 = input!("Enter message: ");
                    broadcast(codes::client::MESSAGE, &server, &inp2);
                }
                4 => {
                    let s1 = server.lock().unwrap();
                    let s2 = server.lock().unwrap();
                }
                _ => println!("Invalid Input"),
            },
            Err(_) => {
                println!("Invalid input");
            }
        }
    }
}
