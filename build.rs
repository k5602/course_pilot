use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/tailwind.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=src/");
    
    // Ensure the assets directory exists
    let assets_dir = "assets";
    fs::create_dir_all(assets_dir).expect("Failed to create assets directory");
    
    // Get the current directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {}", current_dir.display());
    
    // Build the CSS with Tailwind
    let output = Command::new("npx")
        .args([
            "@tailwindcss/cli",
            "-i",
            "./assets/tailwind.css",
            "-o",
            "./assets/tailwind.out.css",
            "--minify"
        ])
        .current_dir(&current_dir)
        .output()
        .expect("Failed to execute tailwindcss command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("tailwindcss failed with status: {}", output.status);
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", stderr);
        panic!("tailwindcss build failed");
    }

    // Verify the output file was created
    let output_path = Path::new(assets_dir).join("tailwind.out.css");
    if !output_path.exists() {
        panic!("tailwind.out.css was not generated at {:?}", output_path);
    }
    
    println!("✅ Tailwind CSS build completed successfully at {:?}", output_path);
    
    // Copy the CSS file to the target directory for development
    let target_dir = current_dir.join("target/debug/assets");
    fs::create_dir_all(&target_dir).expect("Failed to create target assets directory");
    
    let target_css = target_dir.join("tailwind.out.css");
    fs::copy(&output_path, &target_css).expect("Failed to copy CSS to target directory");
    println!("✅ Copied CSS to {:?}", target_css);
}