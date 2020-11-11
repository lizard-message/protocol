use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server decode pub", |b| {
        let mut decode = Decode::new(0);

        b.iter(|| {
            decode.set_buff(&[8, 4]);
            decode.set_buff(b"test");
            decode.set_buff(u32::to_be_bytes(6));
            decode.set_buff(b"qweasd");

            if let Message::Pub(r#pub) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&r#pub.name, &b"test"[..]);
                assert_eq!(&r#pub.msg, &b"qweasd"[..]);
            }
        });
    });

    c.bench_function("server decode pub max", |b| {
        use std::u16::MAX as u16_max;
        use std::u32::MAX as u32_max;
        use std::u8::MAX as u8_max;
        let mut decode = Decode::new(0);

        b.iter(|| {
            decode.set_buff(&[8]);
            decode.set_buff(&[u8_max]);
            decode.set_buff(&[b' '; u8_max as usize]);
            decode.set_buff(u32::to_be_bytes((u16_max << 1) as u32));
            decode.set_buff(&[b' '; (u16_max << 1) as usize]);

            if let Message::Pub(r#pub) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&r#pub.name, &[b' '; u8_max as usize][..]);
                assert_eq!(&r#pub.msg, &[b' '; (u16_max << 1) as usize][..]);
            }
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
