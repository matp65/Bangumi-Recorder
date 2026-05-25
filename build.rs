use std::process::Command;

fn main() {
    // ---- metadata ----
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    println!(
        "cargo:rustc-env=BUILD_GIT_VERSION={}",
        if dirty { format!("{}+dirty", git_hash) } else { git_hash }
    );

    let rustc_version = Command::new("rustc")
        .args(["--version"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={}", rustc_version);

    // ---- build frontend ----
    let frontend = std::path::Path::new("frontend");
    if frontend.exists() {
        // Always re-run if anything in frontend/ changes
        println!("cargo:rerun-if-changed=frontend/");

        // Ensure node_modules exist
        if !frontend.join("node_modules").exists() {
            let status = Command::new("npm")
                .args(["install"])
                .current_dir(frontend)
                .status()
                .expect("npm install failed — is Node.js installed?");
            assert!(status.success(), "npm install failed");
        }

        let status = Command::new("npm")
            .args(["run", "build"])
            .current_dir(frontend)
            .status()
            .expect("npm run build failed");
        assert!(status.success(), "frontend build failed");
    }
}
