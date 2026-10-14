const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let packed = chunk.iter().enumerate().fold(0u32, |packed, (index, byte)| packed | (*byte as u32) << (16 - 8 * index));
        for index in 0..4 {
            out.push(if index <= chunk.len() { ALPHABET[((packed >> (18 - 6 * index)) & 63) as usize] as char } else { '=' });
        }
    }
    out
}
