use crate::common::{U16_SIZE, U32_SIZE, U8_SIZE};
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
pub struct Info {
    pub version: u8,
    pub support: u16,
    pub max_message_length: u32,
}

#[derive(Debug)]
pub struct Erro {
    pub msg: BytesMut,
}

#[derive(Debug)]
pub enum Message {
    Info(Box<Info>),
    Ping,
    Pong,
    TurnPush,
    TurnPull,
    Ok,
    Err(Box<Erro>),
}

#[derive(Debug)]
enum Transition {}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ClientState>,
    length: usize,
}

impl Decode {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            state: None,
            length: 0,
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
        self.length = 0;
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
                            return Some(Ok(Message::Info(Box::new(Info {
                                version: self.source.buffer.get_u8(),
                                support: self.source.buffer.get_u16(),
                                max_message_length: self.source.buffer.get_u32(),
                            }))));
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
                        if self.source.buffer.len() >= U16_SIZE {
                            self.source.length = self.source.buffer.get_u16() as usize;
                            self.source.state = Some(ClientState::ErrContent);
                        } else {
                            return None;
                        }
                    }
                    ClientState::ErrContent => {
                        if self.source.buffer.len() >= self.source.length {
                            let msg = self.source.buffer.split_to(self.source.length);
                            self.source.reset();
                            return Some(Ok(Message::Err(Box::new(Erro { msg }))));
                        } else {
                            return None;
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
