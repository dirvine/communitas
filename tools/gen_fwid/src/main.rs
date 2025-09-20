use anyhow::Result;
use rand::RngCore;
use rand::rngs::OsRng;
use saorsa_core::address::NetworkAddress;
use std::net::Ipv4Addr;

const GENERATION_ATTEMPTS: usize = 1000;

fn generate_valid_four_words() -> Result<String> {
    let mut rng = OsRng;
    const MIN_PORT: u16 = 1024;
    const PORT_SPAN: u32 = u16::MAX as u32 - MIN_PORT as u32 + 1;

    for _ in 0..GENERATION_ATTEMPTS {
        let ipv4 = Ipv4Addr::from(rng.next_u32());
        let port = (rng.next_u32() % PORT_SPAN) as u16 + MIN_PORT;
        let candidate = NetworkAddress::from_ipv4(ipv4, port);

        if let Some(words) = candidate.four_words() {
            // Parse to ensure it's valid
            if let Ok(parsed) = saorsa_core::identity::FourWordAddress::parse_str(words) {
                let words_array: [String; 4] = parsed.words().try_into().map_err(|_| {
                    anyhow::anyhow!("Should have exactly 4 words")
                })?;

                // Validate with saorsa-core
                if saorsa_core::fwid::fw_check(words_array) {
                    return Ok(words.to_string());
                }
            }
        }
    }

    Err(anyhow::anyhow!("Failed to generate valid four-word address after {} attempts", GENERATION_ATTEMPTS))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let count = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(5)
    } else {
        5
    };

    println!("Generating {} valid four-word addresses using saorsa-core v0.3.22:", count);
    println!();

    for i in 1..=count {
        match generate_valid_four_words() {
            Ok(words) => println!("{}: {}", i, words),
            Err(e) => eprintln!("Failed to generate address {}: {}", i, e),
        }
    }

    Ok(())
}