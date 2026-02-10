#[cfg(not(feature = "uniffi-cli"))]
fn main() {
    eprintln!("This binary requires the 'uniffi-cli' feature to be enabled.");
    std::process::exit(1);
}

#[cfg(feature = "uniffi-cli")]
fn main() {
    uniffi::uniffi_bindgen_main()
}
