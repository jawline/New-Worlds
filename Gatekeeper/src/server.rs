use connection::Connection;

use std::io;
use std::str::from_utf8;
use std::io::{Error, ErrorKind};

use world_lib::Map;
use world_lib::message::{next_io, Message};

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

/**
 * Sending u8 buffers logic
 */

impl Server {
    fn send_buffer(&mut self, token: Token, buffer: &[u8]) {
        self.find_connection_by_token(token).send_message(buffer);
    }

    fn send_all_buffer(&mut self, buffer: &[u8], event_loop: &mut EventLoop<Server>) {

        let mut bad_connections = Vec::new();

        for conn in self.conns.iter_mut() {
            conn.send_message(buffer);
            if conn.reregister(event_loop).is_err() {
                bad_connections.push(conn.token);
            }
        }

        for token in bad_connections {
            self.reset_connection(event_loop, token);
        }
    }
}

/**
 * Local for sending Message structs
 */

impl Server {
    fn send_message(&mut self, token: Token, message: &Message) {
        self.send_buffer(token, (message.as_json() + "\0").as_bytes())
    }

    fn broadcast_message(&mut self, message: &Message, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        self.send_all_buffer(format!("{}\0", message.as_json()).as_bytes(), event_loop);
        Ok(())
    }
}

/**
 * Sending message logic
 */

impl Server {
    fn say(&mut self, token: Token, message: &str) {
        self.send_message(token, &Message::Say(message.to_string()))
    }

    fn say_all(&mut self, msg: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        self.broadcast_message(&Message::Say(msg.to_string()), event_loop) 
    }
}

impl Server {
    fn new_connection_accepted(&mut self, _: &mut EventLoop<Server>, _: Token) {
        println!("Accepted new connection");
    }

    fn handle_user_leaving(&mut self, event_loop: &mut EventLoop<Server>, name: &str) -> io::Result<()> {
        self.say_all(&(name.to_string() + " dissolved away"), event_loop)
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

        match self.conns.insert_with(|token| Connection::new(sock, token)) {
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

    fn handshake(&mut self, token: Token, message: Message, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        match message {
            Message::Login(username, _) => {
                self.find_connection_by_token(token).user.set_name(&username);
                let map_json = self.map.as_json();
                println!("Sending JSON");
                self.send_message(token, &Message::Map(map_json));
                self.say_all(&format!("{} has joined the server", username), event_loop)
            },
            _ => { self.kill(token, "Bad login") }
        }
    }

    fn client_message(&mut self, token: Token, message: Message, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        match message {
            Message::Say(msg) => {
                let msg = self.find_connection_by_token(token).user.name.to_string() + ": " + &msg;
                self.say_all(&msg, event_loop)
            },
            Message::Map(mapdata) => {
                self.map = Map::from_json(&mapdata);
                let new_json = self.map.as_json();
                self.broadcast_message(&Message::Map(new_json), event_loop)
            },
            _ => Err(Error::new(ErrorKind::Other, "Unhandled Message"))
        }
    }

    fn kill(&mut self, token: Token, message: &str) -> io::Result<()> {
        self.send_message(token, &Message::Kill(message.to_string()));
        Err(Error::new(ErrorKind::Other, "Killed Connection"))
    }

    fn handle_message(&mut self, token: Token, message: Message, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
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
                let mut local_buffer_copy = (self.find_connection_by_token(token).buffer.to_string() + base_string).to_string();
                while let Some((msg, remain)) = try!(next_io(&local_buffer_copy)) {
                    try!(self.handle_message(token, msg, event_loop));
                    local_buffer_copy = remain;
                }
                self.find_connection_by_token(token).buffer = local_buffer_copy;
                Ok(())
            },
            Err(_) => Err(Error::new(ErrorKind::Other, "corrupted message could not be parsed as utf8"))
        }
    }

    fn read_from_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) -> io::Result<()> {
        println!("server conn readable; token={:?}", token);
        self.is_message(event_loop, token)
    }

    fn reset_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) {
        if self.token == token {
            println!("Server connection reset; shutting down");
            event_loop.shutdown();
        } else {
            
            println!("reset connection; token={:?}", token);
            
            if self.find_connection_by_token(token).write_remaining().is_err() {
                println!("could not write remaining to client before a reset");
            }

            if self.find_connection_by_token(token).shutdown().is_err() {
                println!("could not shutdown TcpStream before a reset");
            }

            let name = self.user_name(token);
            self.conns.remove(token);

            if self.handle_user_leaving(event_loop, &name).is_err() {
                println!("Error handling user leave");
            }
        }
    }

    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut Connection {
        &mut self.conns[token]
    }
}