use crate::common::{U16_SIZE, U32_SIZE, U64_SIZE, U8_SIZE};
use crate::state::ClientState;
use bytes::{Buf, BytesMut};
use std::convert::{AsRef, TryInto};
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
    pub max_message_length: u32,
}

#[derive(Debug)]
pub struct Erro {
    pub msg: BytesMut,
}

#[derive(Debug)]
pub struct Msg {
    pub offset: u64,
    pub payload: BytesMut,
    pub sub_name: BytesMut,
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
    Msg(Box<Msg>),
}

#[derive(Debug)]
enum Transition {
    None,
    Msg {
        offset: u64,
        payload: BytesMut,
        sub_name: BytesMut,
    },
}

impl Transition {
    fn msg() -> Self {
        Transition::Msg {
            offset: 0,
            payload: BytesMut::new(),
            sub_name: BytesMut::new(),
        }
    }

    fn set_msg_offset(&mut self, offset: u64) {
        if let Transition::Msg {
            offset: non_offset,
            payload: _,
            sub_name: _,
        } = self
        {
            *non_offset = offset;
        }
    }

    fn set_msg_subname(&mut self, sub_name: BytesMut) {
        if let Transition::Msg {
            offset: _,
            payload: _,
            sub_name: non_subname,
        } = self
        {
            *non_subname = sub_name;
        }
    }

    fn set_msg_payload(&mut self, payload: BytesMut) {
        if let Transition::Msg {
            offset: _,
            payload: non_payload,
            sub_name: _,
        } = self
        {
            *non_payload = payload;
        }
    }

    fn return_params(&mut self) -> Result<Message, Error> {
        let mut item = Transition::None;
        swap(self, &mut item);

        match item {
            Self::None => Err(Error::Parse),
            Self::Msg {
                offset,
                payload,
                sub_name,
            } => Ok(Message::Msg(Box::new(Msg {
                offset,
                payload,
                sub_name,
            }))),
        }
    }
}

#[derive(Debug)]
pub struct Decode {
    buffer: BytesMut,
    state: Option<ClientState>,
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
        self.params = Transition::None;
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
                        self.source.reset();
                        return Some(Ok(Message::TurnPush));
                    }
                    ClientState::TurnPull => {
                        self.source.reset();
                        return Some(Ok(Message::TurnPull));
                    }
                    ClientState::Ack => {
                        return None;
                    }
                    ClientState::Msg => {
                        self.source.params = Transition::msg();
                        self.source.state = Some(ClientState::MsgOffset);
                    }
                    ClientState::MsgOffset => {
                        if self.source.buffer.len() >= U64_SIZE {
                            self.source
                                .params
                                .set_msg_offset(self.source.buffer.get_u64());
                            self.source.state = Some(ClientState::MsgSubLength);
                        } else {
                            return None;
                        }
                    }
                    ClientState::MsgSubLength => {
                        if self.source.buffer.len() >= U8_SIZE {
                            self.source.length = self.source.buffer.get_u8() as usize;
                            self.source.state = Some(ClientState::MsgSubName);
                        } else {
                            return None;
                        }
                    }
                    ClientState::MsgSubName => {
                        if self.source.buffer.len() >= self.source.length {
                            self.source
                                .params
                                .set_msg_subname(self.source.buffer.split_to(self.source.length));
                            self.source.state = Some(ClientState::MsgLength);
                        } else {
                            return None;
                        }
                    }
                    ClientState::MsgLength => {
                        if self.source.buffer.len() >= U32_SIZE {
                            self.source.length = self.source.buffer.get_u32() as usize;
                            self.source.state = Some(ClientState::MsgPayload);
                        } else {
                            return None;
                        }
                    }
                    ClientState::MsgPayload => {
                        if self.source.buffer.len() >= self.source.length {
                            let payload = self.source.buffer.split_to(self.source.length);
                            self.source.params.set_msg_payload(payload);
                            let msg = self.source.params.return_params();
                            self.source.reset();
                            return Some(msg);
                        } else {
                            return None;
                        }
                    }
                    ClientState::Offset => {
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
