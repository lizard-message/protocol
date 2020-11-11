use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};
use protocol::send_to_server::encode::UnSub;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server decode unsub", |b| {
        let mut decode = Decode::new(0);
        let sub_name = "test";
        let sub_encode = UnSub::new(sub_name).encode();

        b.iter(|| {
            decode.set_buff(&sub_encode);

            if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&us.name, sub_name.as_bytes());
            }
        });
    });

    c.bench_function("server decode unsub max", |b| {
        let mut decode = Decode::new(0);
        const content: [u8; 255] = [b' '; 255];
        let sub_name = unsafe { std::str::from_utf8_unchecked(&content) };
        let sub_encode = UnSub::new(sub_name).encode();

        b.iter(|| {
            decode.set_buff(&sub_encode);

            if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&us.name, sub_name.as_bytes());
            }
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
