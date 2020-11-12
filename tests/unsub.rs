use bytes::{Buf, BytesMut};
use protocol::{
    send_to_client::decode::{Decode, Message},
    send_to_server::encode::UnSub,
};

#[test]
fn decode_unsub() {
    let info = BytesMut::from("test".as_bytes());
    let mut unsub = UnSub::new();
    unsub.push(info.bytes());
    let mut decode = Decode::new(0);

    decode.set_buff(&unsub.encode());

    if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&us.name_list, &vec![info]);
    }
}

#[test]
fn decode_unsub_chunk() {
    let mut decode = Decode::new(0);
    let info = BytesMut::from("test".as_bytes());

    for _ in 0..10 {
        let info2 = info.clone();
        decode.set_buff(&[9]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[0, 1]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[4]);

        assert!(decode.iter().next().is_none());

        decode.set_buff(info.bytes());
        if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&us.name_list, &vec![info2]);
        }
    }
}

#[test]
fn decode_mulit_unsub() {
    let info = BytesMut::from("hello".as_bytes());
    let info2 = BytesMut::from("world".as_bytes());
    let mut unsub = UnSub::new();
    unsub.push(info.bytes());
    unsub.push(info2.bytes());

    let mut decode = Decode::new(0);
    decode.set_buff(&unsub.encode());

    if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&us.name_list, &vec![info, info2]);
    }
}
