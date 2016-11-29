use connection::Connection;

use map::Map;

use help;

use std::io;
use std::str::from_utf8;
use std::str::FromStr;
use std::usize;
use std::io::{Error, ErrorKind};

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
        debug!("events = {:?}", events);
        assert!(token != Token(0), "[BUG]: Received event for Token(0)");

        if events.is_error() {
            warn!("Error event for {:?}", token);
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_hup() {
            trace!("Hup event for {:?}", token);
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_writable() {
            trace!("Write event for {:?}", token);
            assert!(self.token != token, "Received writable event for Server");
            self.find_connection_by_token(token).write_one()
                .and_then(|_| self.find_connection_by_token(token).reregister(event_loop))
                .unwrap_or_else(|e| {
                    warn!("Write event failed for {:?}, {:?}", token, e);
                    self.reset_connection(event_loop, token);
                });
        }

        if events.is_readable() {
            trace!("Read event for {:?}", token);
            if self.token == token {
                self.accept(event_loop);
            } else {
                self.read_from_connection(event_loop, token)
                    .and_then(|_| self.find_connection_by_token(token).reregister(event_loop))
                    .unwrap_or_else(|e| {
                        warn!("Read event failed for {:?}: {:?}", token, e);
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

    fn broadcast_message(&mut self, message: &str, event_loop: &mut EventLoop<Server>) {
        self.send_all(message.as_bytes(), event_loop);
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

    fn handle_user_leaving(&mut self, event_loop: &mut EventLoop<Server>, name: String) {
        self.broadcast_message(&(name + " dissolved away\n"), event_loop);
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

    fn teleport_player_to_zone(&mut self, token: Token, zone_id: usize, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        let valid_zone = self.map.valid_zone_id(zone_id);
        if valid_zone {
            self.find_connection_by_token(token).user.current_zone = zone_id;
            let name = self.user_name(token);
            let to_zone_name = self.current_zone(token);
            self.broadcast_message(&("User ".to_string() + &name + " has teleported to zone " + &to_zone_name + "\n"), event_loop);
        } else {
            self.send_message(token, &(zone_id.to_string() + &" is not a valid zone ID\n"));
        }
        Ok(())
    }

    fn handshake(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        self.find_connection_by_token(token).user.set_name(&message);
        self.send_welcome(token);
        self.broadcast_message(&format!("{} has joined the server\n", &message), event_loop);
        Ok(())
    }

    fn client_message(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        if message.starts_with("say ") {
            let name = self.user_name(token);
            self.broadcast_message(&(name + ": " + &message[4..] + "\n"), event_loop);
            Ok(())
        } else if message.starts_with("set name ") {
            let name_before = self.user_name(token);
            self.find_connection_by_token(token).user.name = message[9..].to_string();
            self.broadcast_message(&("User ".to_string() + &name_before + " set name to " + &message[9..] + "\n"), event_loop);
            Ok(())
        } else if message == "zone" {
            let current_zone = self.current_zone(token);
            let zone_description = self.zone_description(token);
            self.send_message(token, &format!("You are in {}, {}\n", current_zone, zone_description));
            Ok(())
        } else if message == "help" {
            self.send_message(token, help::get_help_text());
            Ok(())
        } else if message == "zones" {
            self.send_zone_list(token);
            Ok(())
        } else if message.starts_with("teleport to id ") {
            let data = &message[15..];
            match usize::from_str(&data) {
                Ok(id) => {
                    self.teleport_player_to_zone(token, id, event_loop)
                },
                Err(_) => {
                    self.broadcast_message("Somebody was an idiot\n", event_loop);
                    Ok(())
                }
            }
        } else if message.starts_with("teleport to ") {
            let zone = &message[12..];
            let zone_id_op = self.map.find_zone_from_name(zone);
            match zone_id_op {
                Some(zone_id) => self.teleport_player_to_zone(token, zone_id, event_loop),
                None => {
                    let name = self.user_name(token);
                    self.broadcast_message(&(name + " has fumbled a teleport location\n"), event_loop);
                    Ok(())
                }
            }
        } else if message == "logout" {
            self.send_message(token, "Goodbye sweet prince\nDon't come back...\n");
            self.reset_connection(event_loop, token);
            Ok(())
        } else {
            let name = self.user_name(token);
            self.broadcast_message(&(name + " has written some unintelligble gibberish\n"), event_loop);
            Ok(())
        }
    }

    fn handle_message(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        if !self.find_connection_by_token(token).handshake_done {
            self.find_connection_by_token(token).handshake_done = true;
            self.handshake(token, message, event_loop)
        } else {
            self.client_message(token, message, event_loop)
        }
    }

    fn read_from_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) -> io::Result<()> {
        debug!("server conn readable; token={:?}", token);

        let message = try!(self.find_connection_by_token(token).readable());

        match from_utf8(&message) {
            Ok(base_string) => {
                self.handle_message(token, base_string.trim(), event_loop)
            },
            Err(_) => Err(Error::new(ErrorKind::Other, "corrupted message could not be parsed as utf8"))
        }
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
            self.handle_user_leaving(event_loop, name);
        }
    }

    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut Connection {
        &mut self.conns[token]
    }
}