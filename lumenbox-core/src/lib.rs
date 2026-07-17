uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn check_connection() -> String {
    "LumenBox Rust Core Connected!".to_string()
}
