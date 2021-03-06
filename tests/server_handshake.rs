use bytes::{BufMut, BytesMut};
use protocol::send_to_client::decode::{Decode, Error, Info, Message};
use protocol::send_to_server::encode::ClientConfig;

fn init(buff: &[u8]) -> Option<Result<Message, Error>> {
    let mut decode = Decode::new(1024);

    decode.set_buff(&buff);
    decode.iter().next()
}

#[test]
fn decode_hand_shake() {
    let mut client_config = ClientConfig::default();
    client_config.set_version(1);
    client_config.support_push();
    client_config.support_pull();
    client_config.max_task_size(10);

    // a success hand shake
    if let Message::Info(info) = init(&client_config.encode()).unwrap().unwrap() {
        assert_eq!(info.version, 1);
        assert_eq!(info.support, 3);
        assert_eq!(info.max_message_size, 10);
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
    if let Message::Info(info) = init(&buff).unwrap().unwrap() {
        assert_eq!(info.version, 1);
        assert_eq!(info.support, 3);
        assert_eq!(info.max_message_size, 10);
    }
}

#[test]
fn decode_hand_shake_chunk() {
    let mut decode = Decode::new(1024);
    let mut buff = BytesMut::new();

    for _ in 0..100 {
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

        if let Message::Info(info) = result.unwrap().unwrap() {
            assert_eq!(info.version, 1);
            assert_eq!(info.support, 3);
            assert_eq!(info.max_message_size, 10);
            buff.clear();
        }
    }
}
