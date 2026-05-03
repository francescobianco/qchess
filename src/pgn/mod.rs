pub mod parser;
pub mod scanner;

pub use parser::{GameMetadata, GameRef, LoadedGame};
pub use scanner::scan_pgn_files;