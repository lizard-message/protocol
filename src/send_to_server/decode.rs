use crate::state::ClientState;
use bytes::BytesMut;
use std::iter::Iterator;
use std::convert::AsRef;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error")]
    Parse,
}

#[derive(Debug)]
pub enum Message {

}

#[derive(Debug)]
pub struct Decode {
    buff: BytesMut,
    state: Option<ClientState>,
}

impl Decode {
    
    pub fn new(capacity: usize) -> Self {
        Self {
            buff: BytesMut::with_capacity(capacity),
            state: None
        }
    }

    pub fn get_mut_buff(&mut self) -> &BytesMut {
        &mut self.buff
    }

    pub fn set_buff<R>(&mut self, buff: R) where R: AsRef<[u8]> {
        self.buff.extend_from_slice(buff.as_ref())
    }

    pub fn iter(&mut self) -> Iter<'_> {
        Iter {
            source: self
        }
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    source: &'a mut Decode,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Result<Message, Error>;
    
    fn next(&mut self) -> Self::Item {
    
    }
}
