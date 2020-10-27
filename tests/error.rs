use bytes::{BufMut, BytesMut};

#[test]
fn server_decode_error() {
    use protocol::send_to_client::decode::{Decode, Erro, Message};
    let mut buf = BytesMut::new();
    buf.put_u8(10);
    buf.put_u16(12);
    buf.put_slice(b"decode error");

    let mut decode = Decode::new(33);
    decode.set_buff(buf);

    if let Message::Err(erro) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&erro.msg, &b"decode error"[..]);
    }
}

#[test]
fn server_decode_error_chunk() {
    use protocol::send_to_client::decode::{Decode, Message};
    let mut decode = Decode::new(33);

    for _ in 0..100 {
        decode.set_buff(&[10]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[0, 12]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(b"decode error");

        if let Message::Err(erro) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&erro.msg, &b"decode error"[..]);
        }
    }
}

#[test]
fn client_decode_error() {
    use protocol::send_to_server::decode::{Decode, Message};

    let mut buf = BytesMut::new();
    buf.put_u8(10);
    buf.put_u16(12);
    buf.put_slice(b"decode error");

    let mut decode = Decode::new(33);
    decode.set_buff(buf);

    if let Message::Err { msg } = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&msg, &b"decode error"[..]);
    }
}

#[test]
fn client_decode_error_chunk() {
    use protocol::send_to_server::decode::{Decode, Message};
    let mut decode = Decode::new(33);

    for _ in 0..100 {
        decode.set_buff(&[10]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[0, 12]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(b"decode error");

        if let Message::Err { msg } = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&msg, &b"decode error"[..]);
        }
    }
}
