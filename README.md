# rust-irc
This a simple irc-like client and server application in rust, and libraries to support them both


## Table of Contents
* Background
* Installation
* Basic Usage

## Background
`rust-irc` is both a custom irc-like protocol running on TCP, as well as a reference implementation of client and server application. In short, a server instance will listen for an uncapped number of clients to connect, and provides some simple abilities to see rooms and users, as well as logging connections. Client instances may specify the host to connect to and attempt to register a nickname. After successful registration, clients may create/join rooms, list users, rooms, and users in rooms, and send messages to either all joined rooms or any particular joined room. 

 While there does exist rust crates such as [irc](https://crates.io/crates/irc) and [irc-rust](https://crates.io/crates/irc-rust) and [simple-irc-server](https://crates.io/crates/simple-irc-server) these have not been studied by this author so this implementation is original.


### Technical Behaviour and Protocol
For technical behaviour and protocol information, please see [rfc.txt](https://github.com/djwesty/rust-irc/blob/main/rfc.txt)

## Source Code Overview

### `src/lib.rs`
This file contains primarily the unique list of bytecodes used by both the client and the server in the irc implementation. Each message in either direction always start with one of these bytecodes in the stream. Errors are generally followed by a 2nd special error byte code also. This file also contains some shared/re-usable functions for modularity

### `src/server.rs`
The server application runs on localhost by default. This behaves as follows
* Try to start a TCP listener on port 6667 and handle errors
* Create a `Server` in a mutex lock which contains an empty map of users to TCP streams and empty map of rooms to list of users
* Spawn a thread for every new incoming TCP connections
* Ensure the first request from each new TCP connection is a nickname registration. With a nickname registration, add the user to the `Server`. 
* Loop the incoming TCP stream for each user and handle commands by examining the opcode, parsing the arguments, and acting accordingly. There are some special considerations in this main loop, such as looking out for 0 byte streams (drops), making sure the users do not register nicknames again, handling commands with different lengths and formats of arguments, and avoiding deadlock on the `Server`

Overall I am satisfied with the Server application, with a few notes.

I would have liked to have the various routines that manipulate or otherwise interact with the `Server` to have been trait implementation of `Server`. However, I was unable to find a good solution in this direction because of the `Arc<Mutex>` which wraps the server.

Additionally, since there are resource limits a maximum cap of clients connecting to the server could also be implemented in the future.

### `src/client.rs`
The client application will 
* Prompt the user for a valid nickname as well as hostname for the server. 
* Attempt to open a TCP stream to the hostname on port 6667, and register the nickname if successful. If that is successful, the client will clone the stream for reading server responses, and clone the stream again for the 'watchdog' which is intended send a heartbeat and stop the client if responses are not had. 
* The main client loop will prompt the client on stdin for an input command. The input command will be parsed, and validated to be in the proper format, and will send the relevant bytecode and message to the server. The reader thread will parse responses and display user information as necessary, including incoming messages.

Like the server, I am satisfied with the client implementation but there are some aspects that are left to be desired.

The use of stdout/stdin on this project leaves some minor user experience issues on client. This is most noticeable when the client is part way through typing a prompt, and a message from the server is displayed in stdout. In the future, a simple window application for the client with a dedicated input and output section would address this. 

Another element to be desired is de-muxing of the channel streams that are displayed to the client. Ideally more work could be done so that the client could choose to 'show' just one room at a time, and switch between room views with easy. Message that come in on the room not shown, would be stored in memory until displayed. With my approach, all client messages are displayed in the same stdout for simplicity. 

### `src/main.rs`
A simple entry-point which will look for the `s` or `c` command line argument to run the server or client module. 

## Building/ Running
It is required to install the [rustup rust toolchain](https://rustup.rs/)

The following commands are tested to build and run the application on GNU+Linux, and may also work on MacOs. Windows instructions may differ slightly and are not provided.

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

## Testing

## Example with 3 machines

#### Machine 1 - Server 
Recover your hostname or IP address and open port 6667 if behind NAT. Then start the server
```bash
cargo run s
```


#### Machine 2 - Client 1
Start the client
```bash
cargo run c
```
Choose an example nickname (westy)

```
Starting the IRC client. No spaces allowed in nicknames or room names. /help to see available commands
Enter your nickname : westy
```

Enter the server host
```
Enter the server host: 192.168.0.3
Connected to localhost
```

Join a room (cat)
```
/join cat
Joined cat. Current rooms: cat
```

#### Machine 3 - Client 2
Start the client
```bash
cargo run c
```
Choose an example nickname (easty)

```
Starting the IRC client. No spaces allowed in nicknames or room names. /help to see available commands
Enter your nickname : easty
```

Enter the server host
```
Enter the server host: 192.168.0.3
```

Join the same room (cat)
```
/join cat
Joined cat. Current rooms: cat
```
#### Machine 2 - Client 1
Send a message
```
hello!
```
#### Machine 3 - Client 2
See the message
```
[cat]:[westy]: hello!
```


## License
See [`LICENSE.txt`](https://github.com/djwesty/rust-irc/blob/main/LICENSE.txt)

###### Author: David Westgate 