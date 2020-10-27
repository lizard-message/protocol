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
pub struct Info {
    pub version: u8,
    pub support: u16,
    pub max_message_size: u8,
}

#[derive(Debug)]
pub struct Erro {
    pub msg: BytesMut,
}

#[derive(Debug)]
pub struct Pub {
    pub name: BytesMut,
    pub msg: BytesMut,
}

#[derive(Debug)]
pub struct Sub {
    pub name: BytesMut,
    pub reply: bool,
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
    Pub(Box<Pub>),
    Sub(Box<Sub>),
}

// 解析出来的参数暂存
#[derive(Debug)]
enum Transition {
    None,
    Sub { name: BytesMut, reply: bool },
    Pub { name: BytesMut, msg: BytesMut },
}

impl Transition {
    fn sub() -> Self {
        Transition::Sub {
            name: BytesMut::new(),
            reply: false,
        }
    }

    fn set_sub_name(&mut self, name: BytesMut) {
        if let Transition::Sub {
            name: non_name,
            reply: _,
        } = self
        {
            *non_name = name;
        }
    }

    fn set_sub_reply(&mut self, reply: bool) {
        if let Transition::Sub {
            name: _,
            reply: non_reply,
        } = self
        {
            *non_reply = reply;
        }
    }

    fn set_pub_name(&mut self, name: BytesMut) {
        if let Transition::Pub {
            name: non_name,
            msg: _,
        } = self
        {
            *non_name = name;
        }
    }

    fn set_pub_msg(&mut self, msg: BytesMut) {
        if let Transition::Pub {
            name: _,
            msg: non_msg,
        } = self
        {
            *non_msg = msg;
        }
    }

    fn return_params(&mut self) -> Result<Message, Error> {
        let mut item = Transition::None;
        swap(self, &mut item);

        match item {
            Self::None => Err(Error::Parse),
            Self::Sub { name, reply } => Ok(Message::Sub(Box::new(Sub { name, reply }))),
            Self::Pub { name, msg } => Ok(Message::Pub(Box::new(Pub { name, msg }))),
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
                            return Some(Ok(Message::Info(Box::new(Info {
                                version: self.source.buffer.get_u8(),
                                support: self.source.buffer.get_u16(),
                                max_message_size: self.source.buffer.get_u8(),
                            }))));
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
                            return Some(Ok(Message::Err(Box::new(Erro { msg: err_msg }))));
                        } else {
                            return None;
                        }
                    }
                    ServerState::Pub => {
                        if self.source.buffer.len() > 1 {
                            self.source.length = self.source.buffer.get_u8() as usize;
                            self.source.state = Some(ServerState::PubSubName);
                        } else {
                            return None;
                        }
                    }
                    ServerState::PubSubNameLength => {}
                    ServerState::PubSubName => {
                        if self.source.buffer.len() > 3 {
                            self.source.length = self.source.buffer.get_u16() as usize;

                            self.source.state = Some(ServerState::PubMsg);
                        } else {
                            return None;
                        }
                    }
                    ServerState::PubMsgLength => {}
                    ServerState::PubMsg => {
                        if self.source.buffer.len() >= self.source.length {
                            let msg = self.source.buffer.split_to(self.source.length);
                            self.source.params.set_pub_msg(msg);
                            self.source.reset();
                            return Some(self.source.params.return_params());
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
                        self.source.state = Some(ServerState::SubReply);
                    }
                    ServerState::SubReply => {
                        if self.source.buffer.len() >= 1 {
                            self.source.params = Transition::sub();

                            let reply = self.source.buffer.get_u8() == 1;
                            self.source.params.set_sub_reply(reply);

                            self.source.state = Some(ServerState::SubNameLength);
                        } else {
                            return None;
                        }
                    }
                    ServerState::SubNameLength => {
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
