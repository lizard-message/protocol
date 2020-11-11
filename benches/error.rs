use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};
use protocol::send_to_server::encode::Err;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server receiver error", |b| {
        let mut decode = Decode::new(0);
        let content = "decode error";
        let err_encode = Err::new(content).encode();

        b.iter(|| {
            decode.set_buff(&err_encode);

            if let Message::Err(erro) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&erro.msg, content.as_bytes());
            }
        });
    });

    c.bench_function("server receiver u16 max error", |b| {
        let mut decode = Decode::new(0);
        const content: [u8; 65535] = [b' '; 65535];
        let content_str = unsafe { std::str::from_utf8_unchecked(&content) };
        let err_encode = Err::new(content_str).encode();

        b.iter(|| {
            decode.set_buff(&err_encode);

            if let Message::Err(erro) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&erro.msg, content_str.as_bytes());
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
