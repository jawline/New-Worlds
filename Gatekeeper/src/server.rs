use connection::Connection;

use map::Map;

use std::io;
use std::str::from_utf8;
use std::str::FromStr;
use std::usize;
use std::io::{Error, ErrorKind};

use mio::*;
use mio::buf::ByteBuf;
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

    pub fn new(sock: TcpListener, map: Map) -> Server {
        Server {
            sock: sock,
            token: Token(1),
            conns: Slab::new_starting_at(Token(2), 2048),
            map: map
        }
    }

    pub fn register(&mut self, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        event_loop.register_opt(
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
            let conn_send_buf = ByteBuf::from_slice(buffer);
            conn.send_message(conn_send_buf);
            if conn.reregister(event_loop).is_err() {
                bad_connections.push(conn.token);
            }
        }

        for token in bad_connections {
            self.reset_connection(event_loop, token);
        }
    }

    fn new_connection_accepted(&mut self, event_loop: &mut EventLoop<Server>, token: Token) {
        let name = self.find_connection_by_token(token).user.name.clone();
        self.send_welcome(token);
        self.broadcast_message(&format!("{} has joined the server\n", name), event_loop);
    }

    fn send_welcome(&mut self, token: Token) {
        let current_zone = &self.current_zone(token);
        let description = &self.zone_description(token);
        self.send_message(token, &("You find yourself in ".to_string() + current_zone + ", " + description + "\n"));
    }

    fn accept(&mut self, event_loop: &mut EventLoop<Server>) {
        debug!("server accepting new socket");

        let sock = match self.sock.accept() {
            Ok(s) => {
                match s {
                    Some(sock) => sock,
                    None => {
                        error!("Failed to accept new socket");
                        self.reregister(event_loop);
                        return;
                    }
                }
            },
            Err(e) => {
                error!("Failed to accept new socket, {:?}", e);
                self.reregister(event_loop);
                return;
            }
        };

        let start_zone = self.map.start_zone;

        match self.conns.insert_with(|token| {
            debug!("registering {:?} with event loop", token);
            Connection::new(sock, token, start_zone)
        }) {
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

    fn send_message(&mut self, token: Token, message: &str) {
        self.send_token_message(token, ByteBuf::from_slice(message.as_bytes()))
    }

    fn broadcast_message(&mut self, message: &str, event_loop: &mut EventLoop<Server>) {
        self.send_all(message.as_bytes(), event_loop);
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

    fn handle_message(&mut self, token: Token, message: &str, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
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
            let current_zone = self.current_zone(token) + "\n";
            self.send_message(token, &("You are in ".to_string() + &current_zone));
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
            self.reset_connection(event_loop, token);
            Ok(())
        } else {
            self.send_token_message(token, ByteBuf::from_slice(b"Error, unknown command\n"));
            Ok(())
        }
    }

    fn read_from_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) -> io::Result<()> {
        debug!("server conn readable; token={:?}", token);

        let message = try!(self.find_connection_by_token(token).readable());

        if message.remaining() == message.capacity() {
            return Ok(());
        }

        match from_utf8(message.bytes()) {
            Ok(base_string) => {
                self.handle_message(token, base_string.trim(), event_loop)
            },
            Err(_) => Err(Error::new(ErrorKind::Other, "corrupted message could not be parsed as utf8"))
        }
    }

    fn send_token_message(&mut self, token: Token, buffer: ByteBuf) {
        self.find_connection_by_token(token).send_message(buffer);
    }

    fn handle_user_leaving(&mut self, event_loop: &mut EventLoop<Server>, token: Token) {
        let name = self.user_name(token);
        self.broadcast_message(&(name + " dissolved away\n"), event_loop);
    }

    fn reset_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) {

        self.handle_user_leaving(event_loop, token);

        if self.token == token {
            debug!("Server connection reset; shutting down");
            event_loop.shutdown();
        } else {

            debug!("reset connection; token={:?}", token);

            //Send any queued items before shutting down
            if self.find_connection_by_token(token).write_remaining().is_err() {
                debug!("could not write remaining to client before a reset");
            }

            if self.find_connection_by_token(token).shutdown().is_err() {
                error!("could not shutdown TcpStream before a reset");
            }

            self.conns.remove(token);
        }
    }

    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut Connection {
        &mut self.conns[token]
    }
}