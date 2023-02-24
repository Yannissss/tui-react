#![feature(never_type)]

// Internal modules
pub mod command;
pub mod executor;
pub mod prelude;
pub mod react;

// Re-exports
pub use anyhow::Result;
pub use async_stream::stream;
pub mod event {
    pub use crossterm::event::*;
}

// Custom export

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
