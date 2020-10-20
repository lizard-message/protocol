use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server receiver error", |b| {
        let mut decode = Decode::new(0);

        let mut buf = Vec::new();
        buf.extend_from_slice(&[10]);
        buf.extend_from_slice(&[0, 12]);
        buf.extend_from_slice(b"decode error");

        b.iter(|| {
            decode.set_buff(&[
                10, 0, 12, b'd', b'e', b'c', b'o', b'd', b'e', b' ', b'e', b'r', b'r', b'o', b'r',
            ]);

            if let Message::Err { msg } = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&msg, &b"decode error"[..]);
            }
        });
    });

    c.bench_function("server receiver u16 max error", |b| {
        use std::u16::MAX;
        let mut decode = Decode::new(0);

        let mut buf = Vec::new();
        buf.extend_from_slice(&[10]);
        buf.extend_from_slice(&(MAX.to_be_bytes()));
        buf.extend_from_slice(&[b' '; MAX as usize]);

        b.iter(|| {
            decode.set_buff(&buf);

            if let Message::Err { msg } = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&msg, &([b' '; MAX as usize])[..]);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
