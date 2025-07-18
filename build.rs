use std::process::Command;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    // Build protobuf definitions
    tonic_build::compile_protos("proto/dafs.proto")?;
    
    println!("cargo:rerun-if-changed=web/");
    println!("cargo:rerun-if-changed=web/dist/");

    // Build the React app
    let status = Command::new("npm")
        .arg("install")
        .current_dir("web")
        .status()
        .expect("Failed to run npm install in web/");
    assert!(status.success(), "npm install failed");

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("web")
        .status()
        .expect("Failed to run npm run build in web/");
    assert!(status.success(), "npm run build failed");

    // Recursively copy web/dist to target/web-assets
    let dist_dir = Path::new("web/dist");
    let out_dir = Path::new("target/web-assets");
    if out_dir.exists() {
        fs::remove_dir_all(out_dir).expect("Failed to remove old web-assets dir");
    }
    copy_dir_all(dist_dir, out_dir).expect("Failed to copy web/dist to target/web-assets");
    
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
} 