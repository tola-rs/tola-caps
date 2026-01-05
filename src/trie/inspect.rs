//! Runtime inspection of capability sets
//!
//! Allows iterating over all capabilities in a set for debugging.

use super::node::{Empty, Leaf, Node16};
use super::capability::Capability;

/// Runtime inspection of capability sets.
///
/// Allows iterating over all capabilities in a set at runtime.
/// Useful for debugging and logging.
pub trait Inspect {
    /// Calls `f` for each capability in the set with its type name.
    fn inspect<F: FnMut(&'static str)>(&self, f: F);
}

impl Inspect for Empty {
    fn inspect<F: FnMut(&'static str)>(&self, _f: F) {}
}

impl<C: Capability> Inspect for Leaf<C> {
    fn inspect<F: FnMut(&'static str)>(&self, mut f: F) {
        f(core::any::type_name::<C>());
    }
}

/// Inspect impl for Node16 using #[node16(each_slot)]
#[macros::node16(each_slot)]
impl<_Slots_> Inspect for _Node16_
where
    each(_Slots_): Inspect + Default,
{
    fn inspect<F: FnMut(&'static str)>(&self, mut f: F) {
        // This line is repeated 16 times with _Slot_ = N0, N1, ..., NF
        <_Slot_ as Inspect>::inspect(&<_Slot_>::default(), &mut f);
    }
}
