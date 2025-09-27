//! Demo script to showcase the complete DHT identity system

use crate::dht_identity::integration::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Communitas DHT Identity System Demo");
    println!("======================================");
    
    demonstrate_identity_lifecycle().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_demo() {
        let result = demonstrate_identity_lifecycle().await;
        assert!(result.is_ok(), "Demo should run successfully");
    }
}
