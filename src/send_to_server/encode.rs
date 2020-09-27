use crate::state::{Support, STATE_CLIENT_INFO, STATE_PING, STATE_PONG};
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
