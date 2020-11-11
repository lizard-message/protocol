use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::encode::Msg;
use protocol::send_to_server::decode::{Decode, Message};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("client decode msg", |b| {
        let mut decode = Decode::new(0);
        let sub_name = b"test";
        let content = b"qweasd";
        let msg = Msg::new(9, &sub_name[..], &content[..]).encode();

        b.iter(|| {
            decode.set_buff(&msg);

            if let Message::Msg(msg) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&msg.offset, &9);
                assert_eq!(&msg.sub_name, &sub_name[..]);
                assert_eq!(&msg.payload, &content[..]);
            }
        });
    });

    c.bench_function("client decode msg max", |b| {
        let mut decode = Decode::new(0);
        let sub_name = [0u8; 255];
        let content = [0u8; 65535];
        let msg = Msg::new(9, &sub_name[..], &content[..]).encode();

        b.iter(|| {
            decode.set_buff(&msg);

            if let Message::Msg(msg) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&msg.offset, &9);
                assert_eq!(&msg.sub_name, &sub_name[..]);
                assert_eq!(&msg.payload, &content[..]);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
