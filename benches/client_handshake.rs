use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_server::decode::{Decode, Message};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("client handshake", |b| {
        let mut decode = Decode::new(0);

        let buff = [0, 1, 0, 3, 0, 0, 0, 10];

        b.iter(|| {
            decode.set_buff(&buff);

            if let Message::Info {
                version,
                support: mask,
                max_message_length: message_length,
            } = decode.iter().next().unwrap().unwrap()
            {
                assert_eq!(version, 1);
                assert_eq!(mask, 3);
                assert_eq!(message_length, 10);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
