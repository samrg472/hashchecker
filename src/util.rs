pub const BUFFER_SIZE: usize = 1024 * 1024 * 4;

macro_rules! hash_file {
    ($file:expr) => {{
        use sha1::Digest;
        let mut hasher = sha1::Sha1::new();
        let mut buf = vec![0; util::BUFFER_SIZE];
        loop {
            let bytes = $file.read(&mut buf).unwrap();
            if bytes > 0 {
                hasher.update(&buf[0..bytes]);
            } else {
                break;
            }
        }
        hasher.finalize()
    }};
}
