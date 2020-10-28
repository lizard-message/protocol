use bytes::{BufMut, BytesMut};

#[test]
fn server_decode_sub() {
    use protocol::send_to_client::decode::{Decode, Message};
    use protocol::send_to_server::encode::Sub;

    let mut decode = Decode::new(0);
    let sub = Sub::new("test", false);

    decode.set_buff(&sub.encode());


    if let Message::Sub(sub) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&sub.name, &b"test"[..]);
        assert_eq!(sub.reply, false);
    }
}

#[test]
fn server_decode_sub_chunk() {
    use protocol::send_to_client::decode::{Decode, Message};

    let mut decode = Decode::new(0);

    for _ in 0..100 {
        decode.set_buff(&[7]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[1]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[4]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(b"test");
        if let Message::Sub(sub) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&sub.name, &b"test"[..]);
            assert_eq!(sub.reply, true);
        }
    }
}
