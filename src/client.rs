use prompted::input;
use rust_irc::codes::client::*;
use std::io::{ Read, Write};
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
                let message: &[u8] = &buffer[..size];
                process_message(message);
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn process_message(message: &[u8]) {
    if let Ok(text) = String::from_utf8(message.to_vec()) {
        println!("from serv: {}", text);
    }
}

pub fn start() {
    println!("Starting the IRC client");
    let nick: String = input!("Enter your nickname: ");
    // let host: String = input!("Enter the server host: ");
    let host: &str = "localhost";

    if let Ok(mut stream) = TcpStream::connect(host.to_owned() + ":6667") {
        println!("Connected to {}", host);

        //try to register the nickname
        let mut buf: Vec<u8> = vec![0; nick.capacity()];
        buf[0] = REGISTER_NICK;
        for i in 1..nick.len()+1 {
            buf[i] = *nick.as_bytes().get(i - 1).unwrap();
        }
        stream.write(&buf);

        //another stream for reading messages
        let stream_clone: TcpStream = stream.try_clone().expect("Failed to clone stream");
        thread::spawn(move || {
            read_messages(stream_clone);
        });

        loop {
            let cmd: String = input!(":");
            match cmd.trim() {
                "/quit" => {}
                "/list" => {}
                "/msq" => {}
                "/join" => {}
                "/show" => {}
                "/leave" => {}
                "/msg" => {}

                _ => {
                    stream.write(cmd.as_bytes());
                }
            }
        }
    } else {
        println!("Failed to connect to {} with nickname {} ", host, nick);
    }
}
