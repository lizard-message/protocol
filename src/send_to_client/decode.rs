use crate::state::ServerState;
use bytes::{Buf, BytesMut};
use std::convert::AsRef;
use std::convert::TryInto;
use std::iter::Iterator;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error")]
    Parse,
}

#[derive(Debug)]
pub enum Message {
    Info {
        version: u8,
        support: u16,
        max_message_size: u8,
    },
    Ping,
    Pong,
    TurnPush,
    TurnPull,
}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ServerState>,
}

impl Decode {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            state: None,
        }
    }

    pub fn get_mut_buffer(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    pub fn set_buff<R>(&mut self, buff: R)
    where
        R: AsRef<[u8]>,
    {
        self.buffer.extend_from_slice(buff.as_ref())
    }

    pub fn iter(&mut self) -> Iter<'_> {
        Iter { source: self }
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    source: &'a mut Decode,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Result<Message, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.source.buffer.has_remaining() {
                return None;
            } else if let Some(state) = &self.source.state {
                match state {
                    ServerState::ClientInfo => {
                        if self.source.buffer.len() > 3 {
                            self.source.state = None;
                            return Some(Ok(Message::Info {
                                version: self.source.buffer.get_u8(),
                                support: self.source.buffer.get_u16(),
                                max_message_size: self.source.buffer.get_u8(),
                            }));
                        } else {
                            return None;
                        }
                    }
                    ServerState::Ping => {
                        self.source.state = None;
                        return Some(Ok(Message::Ping));
                    }
                    ServerState::Pong => {
                        self.source.state = None;
                        return Some(Ok(Message::Pong));
                    }
                    ServerState::Err => {
                        return None;
                    }
                    ServerState::Msg => {
                        return None;
                    }
                    ServerState::Ack => {
                        return None;
                    }
                    ServerState::Offset => {
                        return None;
                    }
                    ServerState::TurnPull => {
                        self.source.state = None;
                        return Some(Ok(Message::TurnPull));
                    }
                    ServerState::TurnPush => {
                        self.source.state = None;
                        return Some(Ok(Message::TurnPush));
                    }
                }
            } else {
                let byte = self.source.buffer.get_u8();

                match byte.try_into() {
                    Ok(state) => self.source.state = Some(state),
                    Err(e) => return Some(Err(Error::Parse)),
                }
            }
        }
    }
}
