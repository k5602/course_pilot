use std::env;
use std::path::Path;
use std::process::Command;

/// Build script to auto-generate Tailwind CSS output.
fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root = Path::new(&manifest_dir);

    // Re-run build script when these files change.
    println!("cargo:rerun-if-changed=assets/tailwind.css");
    println!("cargo:rerun-if-changed=assets/dx-components-theme.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");

    // Ensure npm dependencies are installed
    if !root.join("node_modules").exists() {
        println!("cargo:warning=node_modules not found, running npm install...");
        let install_status = Command::new("npm")
            .arg("install")
            .current_dir(root)
            .status()
            .expect("Failed to execute npm install (is Node.js installed?)");

        if !install_status.success() {
            panic!("npm install failed with status: {install_status}");
        }
    }

    // Run npm build:css to generate assets/tailwind.out.css
    let status = Command::new("npm")
        .arg("run")
        .arg("build:css")
        .current_dir(root)
        .status()
        .expect("Failed to execute npm (is Node.js installed?)");

    if !status.success() {
        panic!("npm run build:css failed with status: {status}");
    }
}
