use bytes::{BufMut, BytesMut};
use protocol::send_to_client::decode::{Decode, Error, Message};

#[test]
fn decode_error() {
    let mut buf = BytesMut::new();
    buf.put_u8(9);
    buf.put_u16(12);
    buf.put_slice(b"decode error");

    let mut decode = Decode::new(33);
    decode.set_buff(buf);

    if let Message::Err { msg: msg } = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&msg, &b"decode error"[..]);
    }
}

#[test]
fn decode_error_chunk() {
    let mut decode = Decode::new(33);

    for _ in 0..100 {
        decode.set_buff(&[9]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[0, 12]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(b"decode error");

        if let Message::Err { msg: msg } = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&msg, &b"decode error"[..]);
        }
    }
}
