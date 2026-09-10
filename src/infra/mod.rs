//! System-boundary code shared by both frontends: reading/writing the saved
//! haiku store and loading the terminal color theme. Anything that touches
//! the filesystem or the OS environment lives here, not in `domain`.

pub mod store;
pub mod style;
