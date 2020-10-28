use bytes::{BufMut, BytesMut};
use protocol::send_to_client::encode::ServerConfig;
use protocol::send_to_server::decode::{Decode, Error, Message};
use std::u8::MAX as u8_MAX;

fn init(buf: &[u8]) -> Option<Result<Message, Error>> {
    let mut decode = Decode::new(1024);

    decode.set_buff(buf);

    decode.iter().next()
}

#[test]
fn decode_handshake() {
    let mut server_config = ServerConfig::default();
    server_config.set_version(1);
    server_config.support_push();
    server_config.support_pull();
    server_config.max_message_length(10);

    if let Message::Info(info) = init(&server_config.encode()).unwrap().unwrap() {
        assert_eq!(info.version, 1);
        assert_eq!(info.support, 3);
        assert_eq!(info.max_message_length, 10);
    }
}

#[test]
#[should_panic]
fn decode_handshake_error() {
    let mut buf = BytesMut::new();
    buf.put_u8(u8_MAX);
    buf.put_u8(1);
    buf.put_u16(3);
    buf.put_u32(10);

    if let Message::Info(info) = init(&buf).unwrap().unwrap() {
        assert_eq!(info.version, 1);
        assert_eq!(info.support, 3);
        assert_eq!(info.max_message_length, 10);
    }
}

#[test]
fn decode_handshake_chunk() {
    let mut decode = Decode::new(1024);

    for _ in 0..100 {
        decode.set_buff(&[0]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[1]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[0, 3]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[0, 0, 0, 10]);

        if let Message::Info(info) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(info.version, 1);
            assert_eq!(info.support, 3);
            assert_eq!(info.max_message_length, 10);
        }
    }
}
