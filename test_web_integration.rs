// Test file to verify web integration logic
use std::path::PathBuf;
use std::fs;

fn main() {
    // Test the web assets path logic
    let web_assets_paths = [
        PathBuf::from("target/web-assets"),           // Development build
        PathBuf::from("/usr/share/dafs/web-assets"), // Installed package
        PathBuf::from("web-assets"),                 // Relative path
    ];
    
    println!("Testing web assets path detection:");
    for (i, path) in web_assets_paths.iter().enumerate() {
        println!("  {}: {} -> exists: {}", i, path.display(), path.exists());
    }
    
    // Test index.html path detection
    let index_paths = [
        PathBuf::from("target/web-assets/index.html"),           // Development build
        PathBuf::from("/usr/share/dafs/web-assets/index.html"), // Installed package
        PathBuf::from("web-assets/index.html"),                 // Relative path
    ];
    
    println!("\nTesting index.html path detection:");
    for (i, path) in index_paths.iter().enumerate() {
        let exists = path.exists();
        let readable = if exists {
            fs::read_to_string(path).is_ok()
        } else {
            false
        };
        println!("  {}: {} -> exists: {}, readable: {}", i, path.display(), exists, readable);
    }
    
    println!("\nWeb integration test completed!");
} 