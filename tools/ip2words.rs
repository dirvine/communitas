#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! four-word-networking = "2.6"
//! ```

use std::env;
use std::net::IpAddr;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <ip> <port>", args[0]);
        eprintln!("Example: {} 167.71.188.131 443", args[0]);
        std::process::exit(1);
    }

    let ip_str = &args[1];
    let port: u16 = args[2].parse().expect("Invalid port number");

    let ip: IpAddr = ip_str.parse().expect("Invalid IP address");

    match ip {
        IpAddr::V4(ipv4) => {
            let words = four_word_networking::encode_ipv4_port(&ipv4.to_string(), port)
                .expect("Failed to encode IPv4");
            println!("IPv4: {} → {}", ip_str, words.join("-"));
        }
        IpAddr::V6(ipv6) => {
            let words = four_word_networking::encode_ipv6_port(&ipv6.to_string(), port)
                .expect("Failed to encode IPv6");
            println!("IPv6: {} → {}", ip_str, words.join("-"));
        }
    }
}
