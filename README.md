# rust-irc
Here we have a simple irc-like client and server application in rust, and libraries to support them both


## Table of Contents
* Background
* Installation
* Basic Usage

## Background
`rust-irc` is both a custom irc-like protocol running on TCP, as well as a reference implementation of client and server application. While there does exist crates such as [irc](https://crates.io/crates/irc) and [irc-rust](https://crates.io/crates/irc-rust) and [simple-irc-server](https://crates.io/crates/simple-irc-server) these have not (yet) been visited by this author so this implemenation is original.


### Behaviour
#### Server
* A host can run the server application on port 6667
* The server can stop gracefully with `0`
* The server can list connected users with `1`
* The server can list active rooms with `2`
* The server can broadcast a message to all users with `3`
* The server can intentionally double lock with `4` (For testing/demonstration only)
#### Client
* A client can connect to a specified host
* A client can become a user by registering their desired nickname with the server (avoiding collisions)
* A new server has no rooms yet, and a new user is not a part of any room.
* A user may both join and create (as relevant) a room with `/join [room]`
* A user may both leave and destroy (as relevant) a room with `/leave [room]`
* A user may see the nickname of all other connected users with `/users`
* A user may see all rooms in existance with `/rooms`
* A user may see all users in a given room with `/list [room]`
* A user may send a message to a specific room (which they have joined) with `/msg [message]`
* A user may simultanously message all rooms they have joined by simply entering their message, not as a command. 
* A user may stop gracefully with `/quit`

### `src/lib.rs`
This file contains primarily the unique list of bytecodes used by both the client and the server in the irc implementation. Each message in either direction always start with one of these bytecodes in the stream. Errors are generally followed by a 2nd special error byte code also. This file also contains some shared/re-usable functions for modularity

### `src/server.rs`
The server application runs on localhost by default. The runs as follows
* Try to start a TCP listener on port 6667 and handle errors
* Create a `Server` in a mutex lock which contains an empty map of users to TCP streams and empty map of rooms to list of users
* Spawn a thread for every new incoming TCP connections
* Ensure the first request from each new TCP connection is a nickname registration. With a nickname registration, add the user to the `Server`. 
* Loop the incoming TCP stream for each user and handle commands by examing the opcode, parsing the arguments, and acting accordingly. There are some special considerations in this main loop, such as looking out for 0 byte streams (drops), making sure the users do not register nicknames again, handling commands with different lengths and formats of arguments, and avoiding deadlock on the `Server`

I would have liked to have the various routines that manipulate or otherwise interact with the `Server` to have been trait implemnetation of `Server`. However, I was unable to find a good solution in this direction because of the `Arc<Mutex>` which wraps the server.

### `src/client.rs`
The client application will 
* Prompt the user for a valid nickname as well as hostname for the server. 
* Attempt to open a TCP stream to the hostname on port 6667, and register the nickname if successfull. If that is successful, the client will clone the stream for reading server responses, and clone the stream again for the 'watchdog' which is intended send a heartbeat and stop the client if responses are not had. 
* The main client loop will prompt the client on stdin for an input command. The input command will be parsed, and validated to be in the proper format, and will send the relevant bytecode and message to the server. The reader thread will parse responses and display user information as necessary, including incoming messages.

The use of stdout/stdin on this project leaves user experience to be desired on the client. This is most noticible when the client is part way through typing a prompt, and a message from the server is displayed in stdout. In the future, a simple window application for the client with a dedicated input and output section would address this. 

Another element to be desired is de-muxing of the channel streams that are displayed to the client. Ideally more work could be done so that the client could choose to 'show' just one room at a time, and switch between room views with easy. Message that come in on the room not shown, would be cached in memory until displayed. With my approach, all client messages are displayed in the same stdout for simplicity. 

## Building/ Running
To run the client in debug mode
```bash
cargo run c
```
To run the server in debug mode
```bash
cargo run s
```
To build and run the application
 
```bash
cargo build --release
./target/release/rust-irc s #to run the server
./target/release/rust-irc c #to run the client
```
Note: Server applications behind NAT may require forwarding of port 6667 TCP to the host.