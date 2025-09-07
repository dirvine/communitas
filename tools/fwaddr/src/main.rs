use std::env;
use std::net::Ipv4Addr;

// Placeholder word list - replace with actual saorsa-core word list
const WORD_LIST: &[&str] = &[
    "ocean", "forest", "moon", "star", "river", "mountain", "sun", "cloud",
    "sparrow", "candle", "ember", "eagle", "wind", "rain", "thunder", "wolf",
    "pack", "hunt", "bear", "claw", "tree", "snow", "ice", "fire"
];

fn ip_to_words(ip: Ipv4Addr, port: u16) -> String {
    // Convert IP and port to a seed for deterministic word selection
    let ip_bytes = ip.octets();
    let seed = ((ip_bytes[0] as u32) << 24) |
               ((ip_bytes[1] as u32) << 16) |
               ((ip_bytes[2] as u32) << 8) |
               (ip_bytes[3] as u32) ^
               (port as u32);

    // Generate four words deterministically from seed
    let word1 = WORD_LIST[(seed % WORD_LIST.len() as u32) as usize];
    let word2 = WORD_LIST[((seed >> 8) % WORD_LIST.len() as u32) as usize];
    let word3 = WORD_LIST[((seed >> 16) % WORD_LIST.len() as u32) as usize];
    let word4 = WORD_LIST[((seed >> 24) % WORD_LIST.len() as u32) as usize];

    format!("{}-{}-{}-{}", word1, word2, word3, word4)
}

fn main() {
    let ip_str = env::args().nth(1).expect("Usage: fwaddr <ipv4> <port>");
    let port: u16 = env::args().nth(2).expect("port").parse().unwrap();

    let ip: Ipv4Addr = ip_str.parse().expect("Invalid IPv4 address");
    let words = ip_to_words(ip, port);
    println!("{}", words);
}