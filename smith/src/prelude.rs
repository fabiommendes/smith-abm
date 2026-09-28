use std::fmt::Debug;

///////////////////////////////////////////////////////////////////////////////
/// TYPE ALIASES
///////////////////////////////////////////////////////////////////////////////

/// Basic representation of time. This crate usually assumes time is measured
/// in days.
pub type Time = u32;

/// Base Real type used by this crate. Uses an alias to easily change precision
/// if necessary.   
pub type Real = f64;
pub(crate) const INF: Real = Real::INFINITY;
pub(crate) const NAN: Real = Real::NAN;

/// Type alias describing agent handles.
pub type Id = usize;

/// State of a single agent
pub trait State: Send + Sync + Clone + PartialEq + Debug + Default {}

///////////////////////////////////////////////////////////////////////////////
/// AGENT TRAITS
///////////////////////////////////////////////////////////////////////////////

/// Agent is just an opaque state with an Id handle. There are no trait bounds
/// and you usually should implement functionality into the state rather than
/// directly on agents.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Agent<S: State> {
    pub id: Id,
    pub state: S,
}
