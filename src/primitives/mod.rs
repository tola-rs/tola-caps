//! # Layer 0: Primitives
//!
//! Basic building blocks for the capability system:
//! - `bool.rs`: Type-level boolean logic (Present/Absent).
//! - `nibble.rs`: Type-level 4-bit values (X0-XF).
//! - `stream.rs`: Infinite hash streams and Peano numbers.

pub mod bool;
pub mod nibble;
pub mod stream;
pub mod identity;
pub mod const_utils;
pub mod pack;
pub mod finger;
pub mod bridge;
pub mod byte_cmp;




// Re-export key types at this level
pub use bool::{Bool, Present, Absent, BoolAnd, BoolOr, BoolNot, SelectBool};
pub use nibble::{Nibble, X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF, NibbleEq};
pub use stream::{HashStream, GetTail, Cons, ConstStream, Z, S, Peano};
