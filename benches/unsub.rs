use criterion::{criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};
use protocol::send_to_server::encode::UnSub;
use bytes::BytesMut;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server decode unsub", |b| {
        let mut decode = Decode::new(0);
        let sub_name = "test";
        let mut unsub = UnSub::new();
        
        unsub.push(sub_name.as_bytes());
        let sub_encode = unsub.encode();

        let info = vec![BytesMut::from(sub_name.as_bytes())];
        b.iter(|| {
            decode.set_buff(&sub_encode);

            if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&us.name_list, &info);
            }
        });
    });

    c.bench_function("server decode unsub max", |b| {
        let mut decode = Decode::new(0);
        const content: [u8; 255] = [b' '; 255];
        
        let mut unsub = UnSub::new();
        unsub.push(&content);
        unsub.push(&content);
        let sub_encode = unsub.encode();

        let info = vec![BytesMut::from(&content[..]), BytesMut::from(&content[..])];

        b.iter(|| {
            decode.set_buff(&sub_encode);

            if let Message::UnSub(us) = decode.iter().next().unwrap().unwrap() {
                assert_eq!(&us.name_list, &info);
            }
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
