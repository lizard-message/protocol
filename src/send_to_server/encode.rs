use crate::state::{
    Support, STATE_CLIENT_INFO, STATE_ERR, STATE_OK, STATE_PING, STATE_PONG, STATE_TURN_PULL,
    STATE_TURN_PUSH,
    STATE_SUB,
    STATE_PUB,
};
use bytes::{BufMut, BytesMut};
use std::default::Default;
use std::u8::MAX as u8_MAX;

#[derive(Debug)]
pub struct ClientConfig {
    version: u8,
    support: u16,
    max_task_size: u8,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            version: 1,
            support: 0,
            max_task_size: u8_MAX,
        }
    }
}

impl ClientConfig {
    pub fn set_version(&mut self, version: u8) {
        self.version = version;
    }

    pub fn support_push(&mut self) {
        self.support |= Support::Push;
    }

    pub fn support_pull(&mut self) {
        self.support |= Support::Pull;
    }

    pub fn support_tls(&mut self) {
        self.support |= Support::Tls;
    }

    pub fn support_compress(&mut self) {
        self.support |= Support::Compress;
    }

    pub fn max_task_size(&mut self, max_task_size: u8) {
        self.max_task_size = max_task_size;
    }

    pub fn encode(self) -> BytesMut {
        let mut buff = BytesMut::with_capacity(5);

        buff.put_u8(STATE_CLIENT_INFO);
        buff.put_u8(self.version);
        buff.put_u16(self.support);
        buff.put_u8(self.max_task_size);

        buff
    }
}

#[derive(Debug)]
pub struct Ping {}

impl Ping {
    pub const fn encode() -> &'static [u8] {
        &[STATE_PING]
    }
}

#[derive(Debug)]
pub struct Pong {}

impl Pong {
    pub const fn encode() -> &'static [u8] {
        &[STATE_PONG]
    }
}

#[derive(Debug)]
pub struct TurnPush {}

impl TurnPush {
    pub const fn encode() -> &'static [u8] {
        &[STATE_TURN_PUSH]
    }
}

#[derive(Debug)]
pub struct TurnPull {}

impl TurnPull {
    pub const fn encode() -> &'static [u8] {
        &[STATE_TURN_PULL]
    }
}

#[derive(Debug)]
pub struct Ok {}

impl Ok {
    pub const fn encode() -> &'static [u8] {
        &[STATE_OK]
    }
}

#[derive(Debug)]
pub struct Err {
    msg: &'static str,
}

impl Err {
    pub fn new(msg: &'static str) -> Self {
        debug_assert!(msg.len() < (std::u16::MAX as usize));
        Self { msg }
    }

    pub fn encode(self) -> BytesMut {
        let mut buff = BytesMut::with_capacity(self.msg.len() + 2 + 1);
        buff.put_u8(STATE_ERR);
        buff.put_u16(self.msg.len() as u16);
        buff.extend_from_slice(self.msg.as_bytes());
        buff
    }
}

#[derive(Debug)]
pub struct Sub<'a> {
    name: &'a str,
}

impl<'a> Sub<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
        }
    }

    pub fn encode(self) -> BytesMut {
        let mut buff = BytesMut::with_capacity(self.name.len() + 3);

        buff.put_u8(STATE_SUB);
        buff.put_u8(self.name.as_bytes().len() as u8);
        buff.extend_from_slice(self.name.as_bytes());

        buff
    }
}

#[derive(Debug)]
pub struct Pub<'a, A> where A: AsRef<[u8]> {
    sub_name: &'a str,
    payload: A
}

impl<'a, A> Pub<'a, A> where A: AsRef<[u8]> {
    pub fn new(sub_name: &'a str, payload: A) -> Self {
        Self {
            sub_name,
            payload
        }
    }

    pub fn encode(self) -> BytesMut {
        let mut buff = BytesMut::new();

        buff.put_u8(STATE_PUB);
        buff.put_u8(self.sub_name.as_bytes().len() as u8);
        buff.extend_from_slice(self.sub_name.as_bytes());

        buff.put_u32(self.payload.as_ref().len() as u32);
        buff.extend_from_slice(self.payload.as_ref());

        buff
    }
}
