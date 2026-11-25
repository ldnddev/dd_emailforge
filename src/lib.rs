//! Email template model, MJML emit, validation, and the terminal UI.
//! The window crate calls this library. It does not grow a second model.

pub mod border;
pub mod cli;
pub mod emit;
pub mod mjml;
pub mod model;
pub mod padding;
pub mod paths;
pub mod preview;
pub mod starters;
pub mod storage;
pub mod tui;
pub mod validate;
