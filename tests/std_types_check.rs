use tola_caps::prelude::*;

#[test]
fn test_std_types_caps() {
    // std::fs::File implements Read, Write, Seek, Debug
    #[cfg(feature = "std")]
    {
        use std::fs::File;
        // File is Sized (implicit).
        assert!(caps_check!(File: std::io::Read));
        assert!(caps_check!(File: std::io::Write));
        assert!(caps_check!(File: std::io::Seek));
        assert!(caps_check!(File: core::fmt::Debug));

        // Check Metadata
        assert!(caps_check!(std::fs::Metadata: core::clone::Clone));
    }

    // std::sync::Mutex (Generic)
    #[cfg(feature = "std")]
    {
        use std::sync::Mutex;
        // Mutex<i32> is Send, Sync, Debug, Default
        assert!(caps_check!(Mutex<i32>: Send));
        assert!(caps_check!(Mutex<i32>: Sync));
        assert!(caps_check!(Mutex<i32>: core::fmt::Debug));
        assert!(caps_check!(Mutex<i32>: Default));

        // Mutex<Rc<i32>> is NOT Send because Rc is not Send
        use std::rc::Rc;
        assert!(!caps_check!(Mutex<Rc<i32>>: Send));
    }

    // std::thread::JoinHandle
    #[cfg(feature = "std")]
    {
        use std::thread::JoinHandle;
        assert!(caps_check!(JoinHandle<()>: core::fmt::Debug));
    }

    // std::path::PathBuf
    #[cfg(feature = "std")]
    {
        use std::path::PathBuf;
        assert!(caps_check!(PathBuf: core::clone::Clone));
        assert!(caps_check!(PathBuf: core::fmt::Debug));
        // PathBuf should implement Default
        assert!(caps_check!(PathBuf: Default));
    }

    // std::net::TcpStream
    #[cfg(feature = "std")]
    {
        use std::net::TcpStream;
        assert!(caps_check!(TcpStream: std::io::Read));
        assert!(caps_check!(TcpStream: std::io::Write));
    }
}
