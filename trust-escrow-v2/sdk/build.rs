use std::path::PathBuf;

fn main() {
    // Path to the v2 contract IDL (now relative from trust-escrow-v2/sdk/)
    let idl_path = PathBuf::from("../target/idl/trust_escrow_v2.json");

    // Tell cargo to rerun if the IDL changes
    println!("cargo:rerun-if-changed={}", idl_path.display());

    // Verify IDL exists at build time
    if !idl_path.exists() {
        panic!(
            "IDL file not found at {}. Please build the v2 contract first with 'anchor build'",
            idl_path.display()
        );
    }

    println!("cargo:rustc-env=IDL_PATH={}", idl_path.display());
    println!("Found Trust Escrow v2 IDL at {}", idl_path.display());
}
