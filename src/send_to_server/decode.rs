use crate::state::ClientState;
use bytes::BytesMut;
use std::iter::Iterator;

#[derive(Debug)]
pub struct Decode {
    buff: BytesMut,
    state: Option<ClientState>,
}

#[derive(Debug)]
pub struct Iter<'a> {
    source: &'a mut Decode,
}
