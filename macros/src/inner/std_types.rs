//! Standard library type definitions for auto-generation.

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Concrete,
    Generic(&'static str),
}

use TypeKind::*;

/// Primitive types (always available).
pub const PRIMITIVE_TYPES: &[(&str, TypeKind)] = &[
    ("()", Concrete),
    ("bool", Concrete),
    ("char", Concrete),

    // Unsigned integers
    ("u8", Concrete),
    ("u16", Concrete),
    ("u32", Concrete),
    ("u64", Concrete),
    ("u128", Concrete),
    ("usize", Concrete),

    // Signed integers
    ("i8", Concrete),
    ("i16", Concrete),
    ("i32", Concrete),
    ("i64", Concrete),
    ("i128", Concrete),
    ("isize", Concrete),

    // Floats
    ("f32", Concrete),
    ("f64", Concrete),

    // NonZero
    ("core::num::NonZeroU8", Concrete),
    ("core::num::NonZeroU16", Concrete),
    ("core::num::NonZeroU32", Concrete),
    ("core::num::NonZeroU64", Concrete),
    ("core::num::NonZeroU128", Concrete),
    ("core::num::NonZeroUsize", Concrete),
    ("core::num::NonZeroI8", Concrete),
    ("core::num::NonZeroI16", Concrete),
    ("core::num::NonZeroI32", Concrete),
    ("core::num::NonZeroI64", Concrete),
    ("core::num::NonZeroI128", Concrete),
    ("core::num::NonZeroIsize", Concrete),
];

pub const CORE_TYPES: &[(&str, TypeKind)] = &[
    // Arrays
    ("[T; N]", Generic("T, const N: usize")),

    // Pointers and references
    ("&T", Generic("T: ?Sized")),
    ("&mut T", Generic("T: ?Sized")),
    ("*const T", Generic("T: ?Sized")),
    ("*mut T", Generic("T: ?Sized")),

    // Option/Result
    ("Option<T>", Generic("T")),
    ("Result<T, E>", Generic("T, E")),

    // Cell types
    ("core::cell::Cell<T>", Generic("T: Copy")),
    ("core::cell::RefCell<T>", Generic("T: ?Sized")),
    ("core::cell::UnsafeCell<T>", Generic("T: ?Sized")),
    ("core::cell::OnceCell<T>", Generic("T")),

    // Memory types
    ("core::mem::ManuallyDrop<T>", Generic("T")),
    ("core::mem::MaybeUninit<T>", Generic("T")),

    // Pin
    ("core::pin::Pin<T>", Generic("T")),

    // Marker types
    ("core::marker::PhantomData<T>", Generic("T: ?Sized")),
    ("core::marker::PhantomPinned", Concrete),

    // Range types
    ("core::ops::Range<T>", Generic("T")),
    ("core::ops::RangeFrom<T>", Generic("T")),
    ("core::ops::RangeTo<T>", Generic("T")),
    ("core::ops::RangeInclusive<T>", Generic("T")),
    ("core::ops::RangeToInclusive<T>", Generic("T")),
    ("core::ops::RangeFull", Concrete),
    ("core::ops::Bound<T>", Generic("T")),

    // Time
    ("core::time::Duration", Concrete),

    // Numeric wrappers
    ("core::num::Wrapping<T>", Generic("T")),
    ("core::num::Saturating<T>", Generic("T")),

    // Async/Task
    ("core::task::Poll<T>", Generic("T")),
    ("core::task::Waker", Concrete),

    // Atomics
    ("core::sync::atomic::AtomicBool", Concrete),
    ("core::sync::atomic::AtomicI8", Concrete),
    ("core::sync::atomic::AtomicI16", Concrete),
    ("core::sync::atomic::AtomicI32", Concrete),
    ("core::sync::atomic::AtomicI64", Concrete),
    ("core::sync::atomic::AtomicIsize", Concrete),
    ("core::sync::atomic::AtomicU8", Concrete),
    ("core::sync::atomic::AtomicU16", Concrete),
    ("core::sync::atomic::AtomicU32", Concrete),
    ("core::sync::atomic::AtomicU64", Concrete),
    ("core::sync::atomic::AtomicUsize", Concrete),
    ("core::sync::atomic::AtomicPtr<T>", Generic("T")),
];

/// Alloc library types (requires "alloc" feature).
pub const ALLOC_TYPES: &[(&str, TypeKind)] = &[
    // Strings
    ("alloc::string::String", Concrete),
    ("alloc::ffi::CString", Concrete),

    // Smart pointers
    ("alloc::boxed::Box<T>", Generic("T: ?Sized")),
    ("alloc::rc::Rc<T>", Generic("T: ?Sized")),
    ("alloc::sync::Arc<T>", Generic("T: ?Sized")),
    ("alloc::rc::Weak<T>", Generic("T: ?Sized")),
    ("alloc::sync::Weak<T>", Generic("T: ?Sized")),

    // Collections
    ("alloc::vec::Vec<T>", Generic("T")),
    ("alloc::collections::VecDeque<T>", Generic("T")),
    ("alloc::collections::LinkedList<T>", Generic("T")),
    ("alloc::collections::BinaryHeap<T>", Generic("T: Ord")),
    ("alloc::collections::BTreeMap<K, V>", Generic("K: Ord, V")),
    ("alloc::collections::BTreeSet<T>", Generic("T: Ord")),

    // Cow
    ("alloc::borrow::Cow<'static, str>", Concrete),
];

/// Generate impl_auto_caps! calls for all types.
pub fn expand_impl_std_types_macro() -> TokenStream {
    let mut impls = Vec::new();

    // Primitives
    for (path, _kind) in PRIMITIVE_TYPES {
        let ty: TokenStream = path.parse().unwrap();
        impls.push(quote! { impl_auto_caps!(#ty); });
    }

    // Core types
    for (path, kind) in CORE_TYPES {
        let ty: TokenStream = path.parse().unwrap();
        match kind {
            Concrete => {
                impls.push(quote! { impl_auto_caps!(#ty); });
            }
            Generic(g) => {
                let generics: TokenStream = g.parse().unwrap();
                impls.push(quote! { impl_auto_caps!(@generic_no_set [#generics] #ty); });
            }
        }
    }

    quote! {
        /// Implement AutoCaps for all primitive and core types.
        macro_rules! impl_std_types {
            () => {
                #(#impls)*
            };
        }
        pub(crate) use impl_std_types;
    }
}

/// Generate impl_auto_caps! calls for alloc types.
pub fn expand_impl_alloc_types_macro() -> TokenStream {
    let mut impls = Vec::new();

    for (path, kind) in ALLOC_TYPES {
        let ty: TokenStream = path.parse().unwrap();
        match kind {
            Concrete => {
                impls.push(quote! { impl_auto_caps!(#ty); });
            }
            Generic(g) => {
                let generics: TokenStream = g.parse().unwrap();
                impls.push(quote! { impl_auto_caps!(@generic_no_set [#generics] #ty); });
            }
        }
    }

    quote! {
        /// Implement AutoCaps for alloc types (Vec, Box, String, etc).
        macro_rules! impl_alloc_types {
            () => {
                extern crate alloc;
                #(#impls)*
            };
        }
        pub(crate) use impl_alloc_types;
    }
}

/// Standard library types (requires "std" feature).
pub const STD_TYPES: &[(&str, TypeKind)] = &[
    // std::sync
    ("std::sync::Mutex<T>", Generic("T: ?Sized")),
    ("std::sync::RwLock<T>", Generic("T: ?Sized")),
    ("std::sync::Condvar", Concrete),
    ("std::sync::Barrier", Concrete),
    ("std::sync::Once", Concrete),
    ("std::sync::OnceLock<T>", Generic("T")),

    // std::thread
    ("std::thread::Thread", Concrete),
    ("std::thread::JoinHandle<T>", Generic("T")),
    ("std::thread::LocalKey<T>", Generic("T: 'static")),

    // std::fs
    ("std::fs::File", Concrete),
    ("std::fs::Metadata", Concrete),
    ("std::fs::FileType", Concrete),
    ("std::fs::DirEntry", Concrete),
    ("std::fs::Permissions", Concrete),
    ("std::fs::OpenOptions", Concrete),
    ("std::fs::ReadDir", Concrete),

    // std::path
    ("std::path::Path", Concrete),
    ("std::path::PathBuf", Concrete),

    // std::net
    ("std::net::IpAddr", Concrete),
    ("std::net::SocketAddr", Concrete),
    ("std::net::TcpStream", Concrete),
    ("std::net::UdpSocket", Concrete),
    ("std::net::TcpListener", Concrete),
    ("std::net::Ipv4Addr", Concrete),
    ("std::net::Ipv6Addr", Concrete),
    ("std::net::SocketAddrV4", Concrete),
    ("std::net::SocketAddrV6", Concrete),

    // std::process
    ("std::process::Command", Concrete),
    ("std::process::Child", Concrete),
    ("std::process::Stdio", Concrete),
    ("std::process::ExitStatus", Concrete),
    ("std::process::Output", Concrete),

    // std::time
    ("std::time::SystemTime", Concrete),
    ("std::time::Instant", Concrete),

    // std::io
    ("std::io::Cursor<T>", Generic("T")),
    ("std::io::BufReader<R>", Generic("R: std::io::Read + ?Sized")),
    ("std::io::BufWriter<W>", Generic("W: std::io::Write + ?Sized")),
    ("std::io::LineWriter<W>", Generic("W: std::io::Write + ?Sized")),
    ("std::io::Error", Concrete),
    ("std::io::ErrorKind", Concrete),

    // std::ffi
    ("std::ffi::OsStr", Concrete),
    ("std::ffi::OsString", Concrete),
    ("std::ffi::CStr", Concrete),
    // CString is already in ALLOC_TYPES (alloc::ffi::CString == std::ffi::CString)
];

/// Generate impl_auto_caps! calls for std types.
pub fn expand_impl_std_lib_types_macro() -> TokenStream {
    let mut impls = Vec::new();

    for (path, kind) in STD_TYPES {
        let ty: TokenStream = path.parse().expect(&format!("Failed to parse type: {}", path));
        match kind {
            Concrete => {
                impls.push(quote! { impl_auto_caps!(#ty); });
            }
            Generic(g) => {
                let generics: TokenStream = g.parse().expect("Failed to parse generics");
                impls.push(quote! { impl_auto_caps!(@generic_no_set [#generics] #ty); });
            }
        }
    }

    quote! {
        /// Implement AutoCaps for std types (std::fs, std::net, etc).
        macro_rules! impl_std_lib_types {
            () => {
                extern crate std;
                #(#impls)*
            };
        }
        pub(crate) use impl_std_lib_types;
    }
}
