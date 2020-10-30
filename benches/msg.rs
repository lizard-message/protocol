use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::encode::Msg;
use protocol::send_to_server::decode::{Decode, Message};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("client decode msg", |b| {
        let mut decode = Decode::new(0);
        let content = b"qweasd";
        let msg = Msg::new(9, &content[..]).encode();

        b.iter(|| {
            decode.set_buff(&msg);

            if let Message::Msg(msg) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&msg.offset, &9);
                assert_eq!(&msg.payload, &content[..]);
            }
        });
    });

    c.bench_function("client decode msg max", |b| {
        let mut decode = Decode::new(0);
        let content = u32::to_be_bytes( (u16::MAX << 2) as u32 );
        let msg = Msg::new(9, &content[..]).encode();

        b.iter(|| {
            decode.set_buff(&msg);

            if let Message::Msg(msg) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&msg.offset, &9);
                assert_eq!(&msg.payload, &content[..]);
            }
        });
    });
} 

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
