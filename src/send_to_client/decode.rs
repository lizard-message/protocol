use crate::state::ServerState;
use bytes::{Buf, BytesMut};
use std::convert::AsRef;
use std::convert::TryInto;
use std::iter::Iterator;
use std::mem::swap;
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
    Pub {
        msg: BytesMut,
    },
    Sub {
        name: BytesMut,
        reply: bool,
    }
}

// 解析出来的参数暂存
#[derive(Debug)]
enum Transition {
    None,
    Sub {
        name: BytesMut,
        reply: bool,
    }
}

impl Transition {
    fn sub() -> Self {
        Transition::Sub {
            name: BytesMut::new(),
            reply: false,
        }
    }

    fn set_sub_name(&mut self, name: BytesMut) {
        if let Transition::Sub{ name: non_name, reply: _} = self {
            *non_name = name;
        }
    }

    fn set_sub_reply(&mut self, reply: bool) {
        if let Transition::Sub{ name: _, reply: non_reply} = self {
            *non_reply = reply;
        }
    }

    fn return_params(&mut self) -> Result<Message, Error> {
        let mut item = Transition::None;
        swap(self, &mut item);

        match item {
            Self::None => Err(Error::Parse),
            Self::Sub { name, reply } => Ok(Message::Sub { name, reply }),
        }
    }
}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ServerState>,
    length: usize,
    params: Transition,
}

impl Decode {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            state: None,
            length: 0,
            params: Transition::None,
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
        self.length = 0;
        self.params = Transition::None;
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
                        if self.source.buffer.len() > 1 {
                            self.source.length = self.source.buffer.get_u16() as usize;
                            self.source.state = Some(ServerState::ErrContent);
                        } else {
                            return None;
                        }
                    }
                    ServerState::ErrContent => {
                        if self.source.buffer.len() >= self.source.length {
                            let err_msg = self.source.buffer.split_to(self.source.length);
                            self.source.reset();
                            return Some(Ok(Message::Err { msg: err_msg }));
                        } else {
                            return None;
                        }
                    }
                    ServerState::Pub => {
                        if self.source.buffer.len() > 3 {
                            self.source.length = self.source.buffer.get_u16() as usize;
                            self.source.state = Some(ServerState::PubMsg);
                        } else {
                            return None;
                        }
                    }
                    ServerState::PubMsg => {
                        if self.source.buffer.len() >= self.source.length {
                            let msg = self.source.buffer.split_to(self.source.length);
                            self.source.reset();
                            return Some(Ok(Message::Pub { msg }));
                        } else {
                            return None;
                        }
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
                    ServerState::Sub => {
                        if self.source.buffer.len() >= 1 {
                            self.source.params = Transition::sub();

                            let reply = self.source.buffer.get_u8() == 1;
                            self.source.params.set_sub_reply(reply);

                            self.source.state = Some(ServerState::SubReply);
                        } else {
                            return None;
                        }
                    }
                    ServerState::SubReply => {
                        if self.source.buffer.len() >= 1 {
                            self.source.length = self.source.buffer.get_u8() as usize;
                            self.source.state = Some(ServerState::SubName);
                        } else {
                            return None;
                        }
                    }
                    ServerState::SubName => {
                        if self.source.buffer.len() >= self.source.length {
                            let sub_name = self.source.buffer.split_to(self.source.length);
                            self.source.params.set_sub_name(sub_name);
                            let message = self.source.params.return_params();
                            self.source.reset();
                            return Some(message);
                        } else {
                            return None;
                        }
                    }
                }
            } else {
                let byte = self.source.buffer.get_u8();
                match byte.try_into() {
                    Ok(state) => self.source.state = Some(state),
                    Err(_) => return Some(Err(Error::Parse)),
                }
            }
        }
    }
}
