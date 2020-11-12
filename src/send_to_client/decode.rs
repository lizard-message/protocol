use crate::common::{U16_SIZE, U32_SIZE, U8_SIZE};
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
}

#[derive(Debug)]
pub struct UnSub {
    pub name_list: Vec<BytesMut>,
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
    UnSub(Box<UnSub>),
}

// 解析出来的参数暂存
#[derive(Debug)]
enum Transition {
    None,
    Sub {
        name: BytesMut,
    },
    Pub {
        name: BytesMut,
        msg: BytesMut,
    },
    UnSub {
        name_list: Vec<BytesMut>,
        total: u16,
        count: u16,
    },
}

impl Transition {
    fn sub() -> Self {
        Transition::Sub {
            name: BytesMut::new(),
        }
    }

    fn set_sub_name(&mut self, sub_name: BytesMut) {
        match self {
            Transition::Sub { name } => {
                *name = sub_name;
            }
            Transition::Pub { name, msg: _ } => {
                *name = sub_name;
            }
            Transition::UnSub {
                name_list,
                total: _,
                count: _,
            } => {
                name_list.push(sub_name);
            }
            _ => {}
        }
    }

    fn r#pub() -> Self {
        Transition::Pub {
            name: BytesMut::new(),
            msg: BytesMut::new(),
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

    fn set_total(&mut self, new_total: u16) {
        match self {
            Self::UnSub {
                name_list: _,
                total,
                count: _,
            } => {
                *total = new_total;
            }
            _ => {}
        }
    }

    fn fetch_add_one(&mut self) {
        match self {
            Self::UnSub {
                name_list: _,
                total: _,
                count,
            } => {
                *count += 1;
            }
            _ => {}
        }
    }

    fn is_enough(&self) -> bool {
        match self {
            Self::UnSub {
                name_list: _,
                total,
                count,
            } => *count >= *total,
            _ => false,
        }
    }

    fn unsub() -> Self {
        Transition::UnSub {
            name_list: Vec::new(),
            total: 0,
            count: 0,
        }
    }

    fn return_params(&mut self) -> Result<Message, Error> {
        let mut item = Transition::None;
        swap(self, &mut item);

        match item {
            Self::None => Err(Error::Parse),
            Self::Sub { name } => Ok(Message::Sub(Box::new(Sub { name }))),
            Self::Pub { name, msg } => Ok(Message::Pub(Box::new(Pub { name, msg }))),
            Self::UnSub {
                name_list,
                total: _,
                count: _,
            } => Ok(Message::UnSub(Box::new(UnSub { name_list }))),
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

    // 统一获取并订阅名称长度
    fn get_and_set_sub_name_length(&mut self) -> Option<()> {
        if self.buffer.len() >= U8_SIZE {
            self.length = self.buffer.get_u8() as usize;
            Some(())
        } else {
            None
        }
    }

    // 按照self.length获取内容
    fn get_payload(&mut self) -> Option<BytesMut> {
        if self.buffer.len() >= self.length {
            Some(self.buffer.split_to(self.length))
        } else {
            None
        }
    }

    // 获取消息数量
    fn get_and_set_total(&mut self) -> Option<()> {
        if self.buffer.len() >= U16_SIZE {
            self.params.set_total(self.buffer.get_u16());
            Some(())
        } else {
            None
        }
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
                        if self.source.buffer.len() >= U32_SIZE {
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
                        if self.source.buffer.len() >= U16_SIZE {
                            self.source.length = self.source.buffer.get_u16() as usize;
                            self.source.state = Some(ServerState::ErrContent);
                        } else {
                            return None;
                        }
                    }
                    ServerState::ErrContent => {
                        let err_msg = self.source.get_payload()?;
                        self.source.reset();
                        return Some(Ok(Message::Err(Box::new(Erro { msg: err_msg }))));
                    }
                    ServerState::Pub => {
                        self.source.params = Transition::r#pub();
                        self.source.state = Some(ServerState::PubSubNameLength);
                    }
                    ServerState::PubSubNameLength => {
                        self.source.get_and_set_sub_name_length()?;
                        self.source.state = Some(ServerState::PubSubName);
                    }
                    ServerState::PubSubName => {
                        let sub_name = self.source.get_payload()?;
                        self.source.params.set_sub_name(sub_name);
                        self.source.state = Some(ServerState::PubMsgLength);
                    }
                    ServerState::PubMsgLength => {
                        if self.source.buffer.len() >= U32_SIZE {
                            self.source.length = self.source.buffer.get_u32() as usize;
                            self.source.state = Some(ServerState::PubMsg);
                        } else {
                            return None;
                        }
                    }
                    ServerState::PubMsg => {
                        let msg = self.source.get_payload()?;
                        self.source.params.set_pub_msg(msg);
                        let message = self.source.params.return_params();
                        self.source.reset();
                        return Some(message);
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
                        self.source.params = Transition::sub();
                        self.source.state = Some(ServerState::SubNameLength);
                    }
                    ServerState::SubNameLength => {
                        self.source.get_and_set_sub_name_length()?;
                        self.source.state = Some(ServerState::SubName);
                    }
                    ServerState::SubName => {
                        let sub_name = self.source.get_payload()?;
                        self.source.params.set_sub_name(sub_name);
                        let message = self.source.params.return_params();
                        self.source.reset();
                        return Some(message);
                    }
                    ServerState::UnSub => {
                        self.source.params = Transition::unsub();
                        self.source.state = Some(ServerState::UnSubTotal);
                    }
                    ServerState::UnSubTotal => {
                        self.source.get_and_set_total()?;
                        self.source.state = Some(ServerState::UnSubNameLength);
                    }
                    ServerState::UnSubNameLength => {
                        self.source.get_and_set_sub_name_length()?;
                        self.source.state = Some(ServerState::UnSubName);
                    }
                    ServerState::UnSubName => {
                        let name = self.source.get_payload()?;
                        self.source.params.set_sub_name(name);
                        self.source.params.fetch_add_one();

                        if self.source.params.is_enough() {
                            let unsub = self.source.params.return_params();
                            self.source.reset();
                            return Some(unsub);
                        } else {
                            self.source.state = Some(ServerState::UnSubNameLength);
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
