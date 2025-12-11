use std::fs;

fn main() {
    // Try to read from .env file
    if let Ok(env_contents) = fs::read_to_string(".env") {
        for line in env_contents.lines() {
            // Skip comments and empty lines
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE format
            if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim();
                let value = line[equals_pos + 1..].trim();

                // Remove quotes if present
                let value = value.trim_matches('"').trim_matches('\'');

                // Set as compile-time environment variable
                if key == "SNAPRAG_API_URL" {
                    println!("cargo:rustc-env=SNAPRAG_API_URL={}", value);
                    println!("cargo:warning=Loaded SNAPRAG_API_URL from .env: {}", value);
                }
                if key == "WALLETCONNECT_PROJECT_ID" {
                    println!("cargo:rustc-env=WALLETCONNECT_PROJECT_ID={}", value);
                    println!(
                        "cargo:warning=Loaded WALLETCONNECT_PROJECT_ID from .env: {}",
                        value
                    );
                }
            }
        }
    }

    // Re-run if .env changes
    println!("cargo:rerun-if-changed=.env");
}
