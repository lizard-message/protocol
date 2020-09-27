use bytes::{BufMut, BytesMut};
use protocol::send_to_server::decode::{Decode, Error, Message};
use std::u8::MAX as u8_MAX;

fn init(buf: &[u8]) -> Option<Result<Message, Error>> {
    let mut decode = Decode::new(1024);

    decode.set_buff(buf);

    decode.iter().next()
}

#[test]
fn decode_handshake() {

    let mut buf = BytesMut::new();
    buf.put_u8(0);

    // version
    buf.put_u8(1);

    // support
    buf.put_u16_le(3);

    // message_length
    buf.put_u32_le(10);

    if let Message::Info(version, mask, max_message_length) = init(&buf).unwrap().unwrap() {
        assert_eq!(version, 1);
        assert_eq!(mask, 3);
        assert_eq!(max_message_length, 10);
    }
}

#[test]
#[should_panic]
fn decode_handshake_error() {
    let mut buf = BytesMut::new();
    buf.put_u8(u8_MAX);
    buf.put_u8(1);
    buf.put_u16_le(3);
    buf.put_u32_le(10);


    if let Message::Info(version, mask, max_message_length) = init(&buf).unwrap().unwrap() {
        assert_eq!(version, 1);
        assert_eq!(mask, 3);
        assert_eq!(max_message_length, 10);
    }
}

#[test]
fn decode_handshake_chunk() {
    let mut decode = Decode::new(1024);

    decode.set_buff(&[0]);
    assert!(decode.iter().next().is_none());

    decode.set_buff(&[1]);
    assert!(decode.iter().next().is_none());

    decode.set_buff(&[3, 0]);
    assert!(decode.iter().next().is_none());

    decode.set_buff(&[10, 0,0,0]);
    
    if let Message::Info(version, mask, max_message_length) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(version, 1);
        assert_eq!(mask, 3);
        assert_eq!(max_message_length, 10);
    }
}
