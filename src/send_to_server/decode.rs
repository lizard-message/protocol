use crate::state::ClientState;
use bytes::{Buf, BytesMut};
use std::convert::{AsRef, TryInto};
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
        max_message_length: u32,
    },
    Ping,
    Pong,
    TurnPush,
    TurnPull,
    Ok,
    Err {
        msg: BytesMut,
    },
}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ClientState>,
    length: Option<usize>,
}

impl Decode {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            state: None,
            length: None,
        }
    }

    pub fn get_mut_buff(&mut self) -> &BytesMut {
        &mut self.buffer
    }

    pub fn set_buff<R>(&mut self, buff: R)
    where
        R: AsRef<[u8]>,
    {
        self.buffer.extend_from_slice(buff.as_ref())
    }

    pub fn reset(&mut self) {
        self.state = None;
        self.length = None;
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
                    ClientState::ServerInfo => {
                        if self.source.buffer.len() > 6 {
                            self.source.reset();
                            return Some(Ok(Message::Info {
                                version: self.source.buffer.get_u8(),
                                support: self.source.buffer.get_u16(),
                                max_message_length: self.source.buffer.get_u32(),
                            }));
                        } else {
                            return None;
                        }
                    }
                    ClientState::Ping => {
                        self.source.reset();
                        return Some(Ok(Message::Ping));
                    }
                    ClientState::Pong => {
                        self.source.reset();
                        return Some(Ok(Message::Pong));
                    }
                    ClientState::TurnPush => {
                        return None;
                    }
                    ClientState::TurnPull => {
                        return None;
                    }
                    ClientState::Ack => {
                        return None;
                    }
                    ClientState::Msg => {
                        return None;
                    }
                    ClientState::Offset => {
                        return None;
                    }
                    ClientState::Sub => {
                        return None;
                    }
                    ClientState::UnSub => {
                        return None;
                    }
                    ClientState::Err => {
                        if self.source.length.is_some() {
                            if let Some(length) = self.source.length.as_ref() {
                                if self.source.buffer.len() >= *length {
                                    let msg = self.source.buffer.split_to(*length);
                                    self.source.reset();
                                    return Some(Ok(Message::Err { msg }));
                                } else {
                                    return None;
                                }
                            }
                        } else {
                            if self.source.buffer.len() > 1 {
                                self.source.length = Some(self.source.buffer.get_u16() as usize);
                            } else {
                                return None;
                            }
                        }
                    }
                    ClientState::Ok => {
                        self.source.reset();
                        return Some(Ok(Message::Ok));
                    }
                }
            } else {
                let byte = self.source.buffer.get_u8();

                match byte.try_into() {
                    Ok(state) => self.source.state = Some(state),
                    Err(_e) => return Some(Err(Error::Parse)),
                }
            }
        }
    }
}
