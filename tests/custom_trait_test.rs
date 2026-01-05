//! Test caps_check with custom trait on concrete types
//!
//! Custom traits work with caps_check! on concrete types WITHOUT any registration.
//! The inline probe pattern detects any trait implementation.

use tola_caps::caps_check;

// Just a regular trait - no #[cap] needed!
pub trait FromEnv {
    fn from_env() -> Self;
}

#[derive(Default, Clone)]
struct DbConfig {
    _host: String,
    _port: u16,
}

impl FromEnv for DbConfig {
    fn from_env() -> Self {
        DbConfig {
            _host: "localhost".to_string(),
            _port: 5432,
        }
    }
}

#[test]
fn test_custom_trait_detection() {
    // caps_check! detects custom traits on concrete types
    assert!(caps_check!(DbConfig: FromEnv));
    assert!(!caps_check!(String: FromEnv));
}
