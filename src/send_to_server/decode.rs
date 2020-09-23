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
    Info(u8, u16, u32),
}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ClientState>,
}

impl Decode {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            state: None,
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
                        if self.source.buffer.len() > 7 {
                            self.source.state = None;
                            return Some(Ok(Message::Info(
                                self.source.buffer.get_u8(),
                                self.source.buffer.get_u16_le(),
                                self.source.buffer.get_u32_le(),
                            )));
                        } else {
                            return None;
                        }
                    }
                    ClientState::Ping => {
                        return None;
                    }
                    ClientState::Pong => {
                        return None;
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
                        return None;
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