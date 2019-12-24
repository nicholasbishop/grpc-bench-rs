pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub fn gen_random_bytes(num_bytes: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(num_bytes);
    for _ in 0..num_bytes {
        bytes.push(rand::random::<u8>());
    }
    bytes
}
