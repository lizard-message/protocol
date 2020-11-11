use protocol::{
  send_to_client::decode::{Decode, Message},
  send_to_server::encode::UnSub,
};

#[test]
fn decode_unsub() {
    let info = "test";
    let unsub = UnSub::new(info);
    let mut decode = Decode::new(0);

    decode.set_buff(&unsub.encode());
    
    if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
        assert_eq!(&us.name, info.as_bytes());
    }
}

#[test]
fn decode_unsub_chunk() {
    let mut decode = Decode::new(0);
    let info = "test";

    for _ in 0..10 {
        decode.set_buff(&[9]);
        assert!(decode.iter().next().is_none());

        decode.set_buff(&[4]);
        
        assert!(decode.iter().next().is_none());

        decode.set_buff(info.as_bytes());
        if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
            assert_eq!(&us.name, info.as_bytes());
        }
    }
}
