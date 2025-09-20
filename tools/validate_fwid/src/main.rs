use anyhow::Result;
use std::env;

fn validate_four_words(input: &str) -> Result<bool> {
    // Parse the four-word address
    let parsed = saorsa_core::identity::FourWordAddress::parse_str(input)?;
    let words_vec = parsed.words();

    if words_vec.len() != 4 {
        return Ok(false);
    }

    let words: [String; 4] = words_vec.try_into().map_err(|_| {
        anyhow::anyhow!("Should have exactly 4 words")
    })?;

    // Validate with saorsa-core
    Ok(saorsa_core::fwid::fw_check(words))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: validate_fwid <four-word-address>");
        eprintln!("Example: validate_fwid philosophy-truth-prevent-wound");
        return Ok(());
    }

    let address = &args[1];

    match validate_four_words(address) {
        Ok(true) => {
            println!("✅ VALID: '{}' passes saorsa-core validation", address);
            std::process::exit(0);
        }
        Ok(false) => {
            println!("❌ INVALID: '{}' fails saorsa-core validation", address);
            std::process::exit(1);
        }
        Err(e) => {
            println!("❌ ERROR: Failed to validate '{}': {}", address, e);
            std::process::exit(1);
        }
    }
}