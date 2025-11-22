use crate::core::{State, StateError};
use std::sync::Arc;

/// Trait for types that can be extracted from the State.
///
/// Implemented for all cloneable types.
/// Use this when you want to extract an owned copy of the value.
pub trait FromState: Sized {
    fn from_state(state: &State) -> Result<Self, StateError>;
}

/// Trait for extracting Arc-wrapped types from State.
///
/// This trait provides zero-cost extraction of Arc<T> from state by
/// borrowing the existing Arc instead of cloning the underlying value.
pub trait FromStateArc: Sized {
    fn from_state_arc(state: &State) -> Result<Self, StateError>;
}

// Implement FromState for cloneable types (uses .get() which clones)
impl<T> FromState for T
where
    T: Clone + Send + Sync + 'static,
{
    fn from_state(state: &State) -> Result<Self, StateError> {
        state.get::<T>()
    }
}

// Implement FromStateArc for Arc<T> (uses .borrow() which returns Arc)
impl<T> FromStateArc for Arc<T>
where
    T: Send + Sync + 'static,
{
    fn from_state_arc(state: &State) -> Result<Self, StateError> {
        state.borrow::<T>()
    }
}
