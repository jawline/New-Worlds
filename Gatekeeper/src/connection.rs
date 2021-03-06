use mio::*;
use mio::tcp::*;

use std::io::Result;
use std::io::{Read, Write};
use std::io::{Error, ErrorKind};
use std::collections::VecDeque;
use user::User;
use world_lib::entity::{null_id, EntityID};
use server::Server;

pub struct Connection {
    pub user: User,
    pub handshake_done: bool,
    pub token: Token,
    pub buffer: Vec<u8>,
    pub entity: EntityID,
    sock: TcpStream,
    interest: EventSet,
    send_queue: VecDeque<Vec<u8>>,
}

impl Connection {
    pub fn new(sock: TcpStream, token: Token) -> Connection {
        Connection {
            user: User::load("Anon", 0),
            sock: sock,
            token: token,
            interest: EventSet::hup(),
            send_queue: VecDeque::new(),
            buffer: Vec::new(),
            handshake_done: false,
            entity: null_id()
        }
    }

    pub fn readable(&mut self) -> Result<Vec<u8>> {
        let mut recv_buf = [0; 2048];

        match self.sock.read(&mut recv_buf) {
            Ok(n) => {
                debug!("CONN : we read {} bytes", n);
                Ok(recv_buf.iter().take(n).map(|&x| x).collect())
            },
            Err(e) => {
                error!("Failed to read buffer for token {:?}, error: {}", self.token, e);
                Err(e)
            }
        }
    }

    pub fn write_one(&mut self) -> Result<()> {
        try!(self.send_queue.pop_front()
            .ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
            .and_then(|buf| {
                match self.sock.write(&buf) {
                    Ok(n) => {
                        debug!("CONN : we wrote {} bytes", n);
                        Ok(())
                    },
                    Err(e) => {
                        error!("Failed to send buffer for {:?}, error: {}", self.token, e);
                        Err(e)
                    }
                }
            })
        );

        if self.send_queue.is_empty() {
            self.interest.remove(EventSet::writable());
        }

        Ok(())
    }

    pub fn write_remaining(&mut self) -> Result<()> {
        while self.interest.is_writable() {
            if let Err(msg) = self.write_one() {
                return Err(msg);
            }
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.sock.shutdown(Shutdown::Both)
    }

    pub fn send_message(&mut self, message: &[u8]) {
        self.send_queue.push_back(message.to_vec());
        self.interest.insert(EventSet::writable());
    }

    pub fn register(&mut self, event_loop: &mut EventLoop<Server>) -> Result<()> {
        self.interest.insert(EventSet::readable());
        event_loop.register(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    pub fn reregister(&mut self, event_loop: &mut EventLoop<Server>) -> Result<()> {
        event_loop.reregister(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }
}