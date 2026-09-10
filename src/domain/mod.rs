//! Pure haiku logic shared by both frontends (`cli`, `tui`) — no I/O, no
//! terminal, no filesystem. Everything here is plain data in, data/Result
//! out, which is what makes it straightforward to unit test.

pub mod haiku;
pub mod import;
pub mod line_check;
pub mod syllables;
