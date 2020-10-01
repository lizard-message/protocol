use bytes::{BufMut, BytesMut};
use protocol::send_to_client::decode::{Decode, Error, Message};

fn init(buff: &[u8]) -> Option<Result<Message, Error>> {
    let mut decode = Decode::new(1024);

    decode.set_buff(&buff);
    decode.iter().next()
}

#[test]
fn decode_hand_shake() {
    let mut buff = BytesMut::new();

    // hand shake
    buff.put_u8(1);

    // version
    buff.put_u8(1);

    // mask
    buff.put_u16(3);

    // client message size
    buff.put_u8(10);

    // a success hand shake
    if let Message::Info {
        version,
        support: mask,
        max_message_size: message_size,
    } = init(&buff).unwrap().unwrap()
    {
        assert_eq!(version, 1);
        assert_eq!(mask, 3);
        assert_eq!(message_size, 10);
    }
}

#[test]
#[should_panic]
fn decode_hand_shake_error() {
    let mut buff = BytesMut::new();

    // hand shake
    buff.put_u8(std::u8::MAX);

    // version
    buff.put_u8(1);

    // mask
    buff.put_u16(3);

    // client message size
    buff.put_u8(10);

    // a success hand shake
    if let Message::Info {
        version,
        support: mask,
        max_message_size: message_size,
    } = init(&buff).unwrap().unwrap()
    {
        assert_eq!(version, 1);
        assert_eq!(mask, 3);
        assert_eq!(message_size, 10);
    }
}

#[test]
fn decode_hand_shake_chunk() {
    let mut decode = Decode::new(1024);
    let mut buff = BytesMut::new();

    buff.put_u8(1);
    decode.set_buff(&buff);
    let result = decode.iter().next();
    dbg!(&result);
    assert!(result.is_none());

    // buffer clear, so not value in buffer
    buff.clear();
    buff.put_u8(1);
    decode.set_buff(&buff);
    let result = decode.iter().next();
    dbg!(&result);
    assert!(result.is_none());

    buff.clear();
    buff.put_u16(3);
    decode.set_buff(&buff);
    let result = decode.iter().next();
    dbg!(&result);
    assert!(result.is_none());

    buff.clear();
    buff.put_u8(10);
    decode.set_buff(&buff);
    let result = decode.iter().next();
    dbg!(&result);

    if let Message::Info {
        version,
        support: mask,
        max_message_size: message_size,
    } = result.unwrap().unwrap()
    {
        assert_eq!(version, 1);
        assert_eq!(mask, 3);
        assert_eq!(message_size, 10);
    }
}
