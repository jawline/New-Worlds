use connection::Connection;

use map::Map;

use help;

use std::io;
use std::str::from_utf8;
use std::str::FromStr;
use std::usize;
use std::io::{Error, ErrorKind};

use world_lib::Message;

use mio::*;
use mio::tcp::*;
use mio::util::Slab;

pub struct Server {
    sock: TcpListener,
    token: Token,
    conns: Slab<Connection>,
    map: Map
}

impl Handler for Server {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Server>, token: Token, events: EventSet) {
        println!("events = {:?}", events);
        println!("{:?} {}", token != Token(0), "[BUG]: Received event for Token(0)");

        if events.is_error() {
            println!("Error event for {:?}", token);
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_hup() {
            println!("Hup event for {:?}", token);
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_writable() {
            println!("Write event for {:?}", token);
            assert!(self.token != token, "Received writable event for Server");
            self.find_connection_by_token(token).write_one()
                .and_then(|_| self.find_connection_by_token(token).reregister(event_loop))
                .unwrap_or_else(|e| {
                    warn!("Write event failed for {:?}, {:?}", token, e);
                    self.reset_connection(event_loop, token);
                });
        }

        if events.is_readable() {
            println!("Read event for {:?}", token);
            if self.token == token {
                self.accept(event_loop);
            } else {
                self.read_from_connection(event_loop, token)
                    .and_then(|_| self.find_connection_by_token(token).reregister(event_loop))
                    .unwrap_or_else(|e| {
                        println!("Read event failed for {:?}: {:?}", token, e);
                        self.reset_connection(event_loop, token);
                    });
            }
        }
    }
}

impl Server {
    pub fn current_zone_id(&mut self, token: Token) -> usize {
        self.find_connection_by_token(token).user.current_zone
    }

    pub fn current_zone(&mut self, token: Token) -> String {
        let current_zone_id = self.current_zone_id(token);
        self.map.zones[current_zone_id].name.clone()
    }

    pub fn zone_description(&mut self, token: Token) -> String {
        let current_zone_id = self.current_zone_id(token);
        self.map.zones[current_zone_id].desc.clone()
    }
}

impl Server {
    fn send_buffer(&mut self, token: Token, buffer: Vec<u8>) {
        self.find_connection_by_token(token).send_message(buffer);
    }

    fn send_message(&mut self, token: Token, message: &str) {
        self.send_buffer(token, message.as_bytes().to_vec())
    }

    fn broadcast_message(&mut self, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        self.send_all(message.as_bytes(), event_loop);
        Ok(())
    }

    fn new_connection_accepted(&mut self, _: &mut EventLoop<Server>, _: Token) {
        //STRIPPED - SEND INIT MESSAGE
        println!("Accepted new connection");
    }

    fn send_welcome(&mut self, token: Token) {
        let current_zone = &self.current_zone(token);
        let description = &self.zone_description(token);
        self.send_message(token, &("You find yourself in ".to_string() + current_zone + ", " + description + "\n"));
    }

    fn send_zone_list(&mut self, token: Token) {
        let mut zone_list = String::new();

        for item in &self.map.zones {
            zone_list = zone_list + &item.id.to_string() + ". " + &item.name + "\n";
        }
        
        self.send_message(token, &zone_list);
    }

    fn handle_user_leaving(&mut self, event_loop: &mut EventLoop<Server>, name: &str) {
        self.say_all(&(name.to_string() + " dissolved away\n"), event_loop);
    }
}

impl Server {

    pub fn new(sock: TcpListener, map: Map) -> Server {
        Server {
            sock: sock,
            token: Token(1),
            conns: Slab::new_starting_at(Token(2), 2048),
            map: map
        }
    }

    pub fn register(&mut self, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        event_loop.register(
            &self.sock,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to register server {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    pub fn reregister(&mut self, event_loop: &mut EventLoop<Server>) {
        event_loop.reregister(
            &self.sock,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).unwrap_or_else(|e| {
            error!("Failed to reregister server {:?}, {:?}", self.token, e);
            let server_token = self.token;
            self.reset_connection(event_loop, server_token);
        })
    }

    fn send_all(&mut self, buffer: &[u8], event_loop: &mut EventLoop<Server>) {

    	let mut bad_connections = Vec::new();

        for conn in self.conns.iter_mut() {
            conn.send_message(buffer.to_vec());
            if conn.reregister(event_loop).is_err() {
                bad_connections.push(conn.token);
            }
        }

        for token in bad_connections {
            self.reset_connection(event_loop, token);
        }
    }

    fn grab_incoming(&mut self) -> Option<TcpStream> {
        match self.sock.accept() {
            Ok(incoming) => match incoming {
                Some((sock, _)) => Some(sock),
                None => {
                    error!("Failed to accept new socket");
                    None
                }
            },
            Err(e) => {
                error!("Failed to accept new socket {}", e);
                None
            }
        }
    }

    fn accept(&mut self, event_loop: &mut EventLoop<Server>) {
        debug!("server accepting new socket");

        let sock = self.grab_incoming();
        self.reregister(event_loop);

        if !sock.is_some() {
            return;
        }

        let sock = sock.unwrap();

        let start_zone = self.map.start_zone;

        match self.conns.insert_with(|token| Connection::new(sock, token, start_zone)) {
            Some(token) => {
                match self.find_connection_by_token(token).register(event_loop) {
                    Ok(_) => {
                        self.new_connection_accepted(event_loop, token);
                    },
                    Err(e) => {
                        error!("Failed to register {:?} connection with event loop, {:?}", token, e);
                        self.conns.remove(token);
                    }
                }
            },
            None => {
                error!("Failed to insert connection into slab");
            }
        };

        self.reregister(event_loop);
    }

    fn user_name(&mut self, token: Token) -> String {
        self.find_connection_by_token(token).user.name.clone()
    }

    fn say_all(&mut self, msg: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        self.broadcast_message(&(Message::Say(msg.to_string()).as_json() + "\0"), event_loop) 
    }

    fn handshake(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        match Message::from_json(message) {
            Ok(Message::Login(username, _)) => {
                self.find_connection_by_token(token).user.set_name(&username);
                self.send_welcome(token);
                self.say_all(&format!("{} has joined the server", username), event_loop)
            },
            _ => { self.kill(token, "Bad login") }
        }
    }

    fn client_message(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        match Message::from_json(message) {
            Ok(Message::Say(msg)) => {
                let msg = self.find_connection_by_token(token).user.name.to_string() + ": " + &msg;
                self.say_all(&msg, event_loop)
            },
            _ => Err(Error::new(ErrorKind::Other, "Unhandled Message"))
        }
    }

    fn kill(&mut self, token: Token, message: &str) -> io::Result<()> {
        self.send_message(token, &Message::Kill(message.to_string()).as_json());
        Err(Error::new(ErrorKind::Other, "Killed Connection"))
    }

    fn handle_message(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        if !self.find_connection_by_token(token).handshake_done {
            self.find_connection_by_token(token).handshake_done = true;
            self.handshake(token, message, event_loop)
        } else {
            self.client_message(token, message, event_loop)
        }
    }

    fn is_message(&mut self, event_loop: &mut EventLoop<Server>, token: Token) -> io::Result<()> {
        let message = try!(self.find_connection_by_token(token).readable());

        match from_utf8(&message) {

            Ok(base_string) => {

                //Add current message to buffer
                self.find_connection_by_token(token).buffer += base_string;

                loop {
                    //Find the null terminator that splits messages
                    let idx = self.find_connection_by_token(token).buffer.find('\0');

                    //Break if no messages left to handle
                    if idx.is_none() {
                        break;
                    }

                    //Mutable outside of closure to avoid borrow hell
                    let (msg, remain): (String, String);

                    //Closure to split the buffer on the token
                    {
                        //Split remain_p to include the null terminator
                        let (msg_p, mut remain_p) = self.find_connection_by_token(token).buffer.split_at(idx.unwrap());

                        //Split the null terminator off of message
                        remain_p = &remain_p[1..];
                        msg = msg_p.to_string();
                        remain = remain_p.to_string();
                    };

                    println!("Handling Message: Msg: {} Remain: {}", msg, remain);

                    self.find_connection_by_token(token).buffer = remain;
                    try!(self.handle_message(token, msg.trim(), event_loop));
                }
                Ok(())
            },
            Err(_) => Err(Error::new(ErrorKind::Other, "corrupted message could not be parsed as utf8"))
        }
    }

    fn read_from_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) -> io::Result<()> {
        debug!("server conn readable; token={:?}", token);
        self.is_message(event_loop, token)
    }

    fn reset_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) {
        if self.token == token {
            debug!("Server connection reset; shutting down");
            event_loop.shutdown();
        } else {
            debug!("reset connection; token={:?}", token);
            
            if self.find_connection_by_token(token).write_remaining().is_err() {
                debug!("could not write remaining to client before a reset");
            }

            if self.find_connection_by_token(token).shutdown().is_err() {
                error!("could not shutdown TcpStream before a reset");
            }

            let name = self.user_name(token);
            self.conns.remove(token);
            self.handle_user_leaving(event_loop, &name);
        }
    }

    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut Connection {
        &mut self.conns[token]
    }
}