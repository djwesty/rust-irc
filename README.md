# rust-irc
This a simple irc-like client and server application in rust, and libraries to support them both

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

The following commands are tested to build and run the application on GNU+Linux, and may also work on MacOS. Windows instructions may differ slightly and are not provided.

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
Testing for this application was done through a combination of manual end-to-end tests and coded unit tests.

### Manual End-to-End tests
Manual end-to-end tests that I have performed provide the most confidence. Taking advantage of the PSU CAT resources, my testing flow was as follows.

#### Client side end-to-end
1) The the server on my local machine in the lab (fab02.cecs.pdx.edu);
2) SSH into 4 seperate clients on the local network (ada, babbage, rita, ruby)
3) Start each client and connect to my server on fab02 with unique nicknames (westy, easty, northy, southy)
4) Have all four clients `/join cat`
5) Have 2 clients also `/join dog` (easty, westy)
6) Test that easty and westy can `/msg dog` and only the other users gets the message
7) Test easty and westy send messages to both `cat` and `dog` when sent in general, ensuring that only the other sees the `dog` room messages, and the 3 other users see the `cat` room messages
8) Test that northy and southy cannot `/msg dog [some message]`
9) Test that northy and southy cannot `/leave dog`
10) Test `/rooms` and `/users` on all clients for expected output
11) Test that a 5th client cannot connect with a duplicate nickname (westy again)
12) Test that all users may `/leave cat` and westy and easty may `/leave dog`, only once
13) Test `/help` message appears
14) Test `/quit` is graceful
15) Test the client side watchdog by running command `4` on the server, having the client send a message and awaiting the client to stop for non-responsiveness.
#### Server side end-to-end
Here, I run the server and have 4 clients connect and join rooms, as in step 1-5 above. Then I test.
1) Test command `1` to see that all 4 users appears
2) Test command `2` to see that all 4 users are in `cat` and westy and easty are in `dog`
3) Test command `3` to see that 'hello world' sent from the server is received by all clients
4) Test command `0` stops the server and gracefully disconnects the clients.

### Coded Unit tests
Coded unit tests are provided for my buffer helper functions and can be found in `tests/test_lib.rs`. I also include an additional test for  `fn remove_user()` in `server.rs`.

I consider my included coded unit tests to be less than adequate and I attribute this to a few reasons. Both the client and server applications rely heavily on buffered reads and writes in every function, whether they be reads from stdin or tpcstream, or writes to stdout or tcpstream. The following work-arounds to this issue were explored but not implemented
1) Re-factor my entire project to use [`Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html) as an abstraction layer to facilitate testing
* This would straightforwardly solve my issue of writing tests for function which take buffers as parameters (or could take buffers, in the case of stdin/stdout). However, I cannot not do this given the effort involved and time restrictions. My situation is a good argument for the test-driven-development paradigm; had I written my tests first, I would have been forced to use `Cursor` from the start.
2) Use a crate like `captured-output` or `gag` to capture stdout and test it
* This would solve 1/4 of the cases listed but since it's not a holistic solution, I'd rather not being in a new external crate *just* for this.
3) use `std::io::set_output_capture` to test stdout
* Again this would solve 1/4 of the cases and looks good as it's in the standard library. However, this is a nightly feature and internet research suggests against using this for stability reasons.
4) Create a mock/dummy TCP listener/ TCP Stream for tests
* Creating a true dummy or mock TCP listener would take some development efforts, not help me with stdout/stdin testing and at that point, I would go back to consider `Cursor`. As a 'quicker' solution in my test for `fn remove_user()` I use a real TCPStream to localhost:6667; this works as its just a placeholder object and not the focus of my test. This is not a reliable approach from an operating system and concurrency perspective, but perhaps I could use `unsafe` raw pointers or refactor my `Server` struct to contain an `Option<TCPStream>` to continue to explore this type of testing further. 

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
Connected to 192.168.0.3:6667
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
Connected to 192.168.0.3:6667
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