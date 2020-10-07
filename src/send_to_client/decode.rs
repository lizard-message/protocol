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
    Ok,
    Err {
        msg: BytesMut,
    },
}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ServerState>,
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

    pub fn get_mut_buffer(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    pub fn set_buff<R>(&mut self, buff: R)
    where
        R: AsRef<[u8]>,
    {
        self.buffer.extend_from_slice(buff.as_ref());
    }

    pub fn iter(&mut self) -> Iter<'_> {
        Iter { source: self }
    }

    // 重置状态和length
    fn reset(&mut self) {
        self.state = None;
        self.length = None;
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
                            self.source.reset();
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
                        self.source.reset();
                        return Some(Ok(Message::Ping));
                    }
                    ServerState::Pong => {
                        self.source.reset();
                        return Some(Ok(Message::Pong));
                    }
                    ServerState::Err => {
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
                        self.source.reset();
                        return Some(Ok(Message::TurnPull));
                    }
                    ServerState::TurnPush => {
                        self.source.reset();
                        return Some(Ok(Message::TurnPush));
                    }
                    ServerState::Ok => {
                        self.source.reset();
                        return Some(Ok(Message::Ok));
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
