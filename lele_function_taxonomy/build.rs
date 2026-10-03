use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let sysroot = Command::new(&rustc)
        .arg("--print")
        .arg("sysroot")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=LELE_TAXONOMY_SYSROOT={sysroot}");
    let src = format!(
        "pub const BUILD_SYSROOT: &str = {};",
        quote::quote!(#sysroot)
    );
    let out =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap_or_default()).join("sysroot.rs");
    let _ = std::fs::write(out, src);
}
