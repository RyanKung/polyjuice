use std::env;
use std::fs;

fn main() {
    // Priority: Environment variables > .env file
    // This allows GitHub Actions to pass secrets via environment variables

    // Check for SNAPRAG_API_URL
    if let Ok(value) = env::var("SNAPRAG_API_URL") {
        println!("cargo:rustc-env=SNAPRAG_API_URL={}", value);
        println!("cargo:warning=Loaded SNAPRAG_API_URL from environment variable");
    } else if let Ok(env_contents) = fs::read_to_string(".env") {
        // Fallback to .env file
        for line in env_contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim();
                let value = line[equals_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                if key == "SNAPRAG_API_URL" {
                    println!("cargo:rustc-env=SNAPRAG_API_URL={}", value);
                    println!("cargo:warning=Loaded SNAPRAG_API_URL from .env: {}", value);
                    break;
                }
            }
        }
    }

    // Check for AUTH_TOKEN
    if let Ok(value) = env::var("AUTH_TOKEN") {
        println!("cargo:rustc-env=AUTH_TOKEN={}", value);
        println!("cargo:warning=Loaded AUTH_TOKEN from environment variable");
    } else if let Ok(env_contents) = fs::read_to_string(".env") {
        for line in env_contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim();
                let value = line[equals_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                if key == "AUTH_TOKEN" {
                    println!("cargo:rustc-env=AUTH_TOKEN={}", value);
                    println!("cargo:warning=Loaded AUTH_TOKEN from .env");
                    break;
                }
            }
        }
    }

    // Check for AUTH_SECRET
    if let Ok(value) = env::var("AUTH_SECRET") {
        println!("cargo:rustc-env=AUTH_SECRET={}", value);
        println!("cargo:warning=Loaded AUTH_SECRET from environment variable");
    } else if let Ok(env_contents) = fs::read_to_string(".env") {
        for line in env_contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim();
                let value = line[equals_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                if key == "AUTH_SECRET" {
                    println!("cargo:rustc-env=AUTH_SECRET={}", value);
                    println!("cargo:warning=Loaded AUTH_SECRET from .env");
                    break;
                }
            }
        }
    }

    // Generate build version file for cache busting
    // Use timestamp in milliseconds as version number
    let build_version = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    // Write version to a file that will be included at compile time
    let out_dir = env::var("OUT_DIR").unwrap();
    let version_file = format!("{}/build_version.txt", out_dir);
    fs::write(&version_file, build_version.to_string())
        .expect("Failed to write build version file");
    println!("cargo:warning=Generated BUILD_VERSION: {}", build_version);
    
    // Force rebuild every time to ensure BUILD_VERSION is always fresh
    println!("cargo:rerun-if-changed=build.rs");

    // Re-run if .env changes
    println!("cargo:rerun-if-changed=.env");
}
