#[cfg(not(feature = "uniffi-bindings"))]
fn main() {
    eprintln!("This binary requires the 'uniffi-bindings' feature to be enabled.");
    std::process::exit(1);
}

#[cfg(feature = "uniffi-bindings")]
fn main() {
    uniffi::uniffi_bindgen_main()
}
