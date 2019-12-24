pub mod bench;
pub mod bench_grpc;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub fn gen_random_bytes() -> Vec<u8> {
    let num_bytes = 2 * 1024 * 1024; // 2 MiB
    let mut bytes = Vec::with_capacity(num_bytes);
    for _ in 0..num_bytes {
        bytes.push(rand::random::<u8>());
    }
    bytes
}
