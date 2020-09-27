use criterion::{black_box, criterion_group, criterion_main, Criterion};
use protocol::send_to_client::decode::{Decode, Message};


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("server handshake", |b| {
       let mut decode = Decode::new(0);

       let buf = [1, 1, 0, 3, 10];

       b.iter(|| {
          decode.set_buff(&buf);

          if let Message::Info(version, mask, task_size) = decode.iter().next().unwrap().unwrap() {
              assert_eq!(version, 1);
              assert_eq!(mask, 3);
              assert_eq!(task_size, 10);
          }
       });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
