// Dev mode
#[cfg(debug_assertions)]
pub const DEV_MODE: bool = true;
#[cfg(not(debug_assertions))]
pub const DEV_MODE: bool = false;

// Server info

pub const SERVER_PORT: u16 = 3000;

#[cfg(debug_assertions)] // debug mode
pub const SERVER_URL: &str = "http://localhost:3000";
#[cfg(not(debug_assertions))] // release mode
pub const SERVER_URL: &str = "https://heapswap.com";

// Misc constants

// time
pub const S: u64 = 1;
pub const MS: u64 = 1000;
pub const US: u64 = 1000 * MS;
pub const NS: u64 = 1000 * US;
pub const PS: u64 = 1000 * NS;

// byte sizes
pub const KB: u64 = 1024;
pub const MB: u64 = KB * KB;
pub const GB: u64 = MB * KB;
pub const TB: u64 = GB * KB;
