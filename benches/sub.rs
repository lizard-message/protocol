use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server decode sub", |b| {
        let mut decode = Decode::new(0);

        b.iter(|| {
            decode.set_buff(&[7, 1, 4]);
            decode.set_buff(b"test");

            if let Message::Sub { name, reply } = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&name, &b"test"[..]);
                assert_eq!(reply, true);
            }
        });
    });

    c.bench_function("server decode sub max", |b| {
        let mut decode = Decode::new(0);

        b.iter(|| {
            decode.set_buff(&[7, 1, 255]);
            decode.set_buff(&[b' '; 255]);

            if let Message::Sub { name, reply } = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&name, &[b' '; 255][..]);
                assert_eq!(reply, true);
            }
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
