use bytes::{BufMut, BytesMut};
use protocol::send_to_client::decode::{Decode, Message};
use protocol::send_to_server::encode::Pub;

#[test]
fn pub_decode() {
    let publish = Pub::new("test", "qweasd");
    let mut decode = Decode::new(0);
    decode.set_buff(publish.encode());

    if let Message::Pub(r#pub) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&r#pub.name, &"test"[..]);
        assert_eq!(&r#pub.msg, &"qweasd"[..]);
    }
}

#[test]
fn pub_decode_chunk() {
    let mut decode = Decode::new(0);
    let mut buff = BytesMut::new();

    for _ in 0..100 {
        buff.put_u8(8);
        decode.set_buff(&buff);
        assert!(decode.iter().next().is_none());
        buff.clear();

        buff.put_u8(4);

        decode.set_buff(&buff);
        assert!(decode.iter().next().is_none());
        buff.clear();

        buff.extend_from_slice(b"test");

        decode.set_buff(&buff);
        assert!(decode.iter().next().is_none());
        buff.clear();

        buff.put_u32(6);

        decode.set_buff(&buff);
        assert!(decode.iter().next().is_none());
        buff.clear();

        buff.extend_from_slice(b"qweasd");
        decode.set_buff(&buff);

        if let Message::Pub(r#pub) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&r#pub.name, &"test"[..]);
            assert_eq!(&r#pub.msg, &"qweasd"[..]);
            buff.clear();
        }
    }
}
