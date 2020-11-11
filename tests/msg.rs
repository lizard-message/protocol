use protocol::send_to_client::encode::Msg;
use protocol::send_to_server::decode::{Decode, Error, Message};

#[test]
fn decode_msg() {
    let msg = Msg::new(9, b"test_msg", b"test");
    let mut decode = Decode::new(0);

    decode.set_buff(&msg.encode());

    if let Message::Msg(msg) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&msg.offset, &9);
        assert_eq!(&msg.sub_name, &b"test_msg"[..]);
        assert_eq!(&msg.payload, &b"test"[..]);
    }
}

#[test]
fn decode_msg_chunk() {
    let mut decode = Decode::new(0);

    for _ in 0..10 {
        decode.set_buff(&[4]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(u64::to_be_bytes(4));
        assert!(decode.iter().next().is_none());

        decode.set_buff(u8::to_be_bytes(4));
        assert!(decode.iter().next().is_none());

        decode.set_buff(b"test");
        assert!(decode.iter().next().is_none());

        decode.set_buff(u32::to_be_bytes(6));
        assert!(decode.iter().next().is_none());

        decode.set_buff(b"qweasd");
        if let Message::Msg(msg) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&msg.offset, &4);
            assert_eq!(&msg.payload, &b"qweasd"[..]);
        }
    }
}
