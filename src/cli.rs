use clap::{Parser, Subcommand};

// SPEC NOTE:
// Inline CLI arguments are assumed to be valid UTF-8 by the OS/CLI layer.
// AnchorScope enforces UTF-8 only for file-based inputs explicitly.

#[derive(Parser)]
#[command(name = "anchorscope", version = "1.1.0", about = "AnchorScope v1.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Find anchor in file and return location + hash.
    Read {
        /// Path to the target file.
        #[arg(long)]
        file: String,

        /// Anchor string (exact, multi-line allowed via escape sequences).
        /// Pass the anchor as a raw argument; use $'...' in shell for \n.
        #[arg(long)]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long)]
        anchor_file: Option<String>,
    },

    /// Replace anchor region if expected_hash matches.
    Write {
        /// Path to the target file.
        #[arg(long)]
        file: String,

        /// Anchor string — must match exactly.
        #[arg(long)]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long)]
        anchor_file: Option<String>,

        /// Expected xxh3 hash (hex) of the matched region.
        #[arg(long)]
        expected_hash: String,

        /// Replacement string (replaces the entire anchor region).
        #[arg(long)]
        replacement: String,
    },

    /// Define a unique labeled region by storing anchor + hash.
    Anchor {
        /// Path to the target file.
        #[arg(long)]
        file: String,

        /// Label/name for this anchor (unique identifier).
        #[arg(long)]
        label: String,

        /// Anchor string.
        #[arg(long)]
        anchor: Option<String>,

        /// Path to file containing anchor.
        #[arg(long)]
        anchor_file: Option<String>,

        /// Expected xxh3 hash (hex) of the matched region.
        #[arg(long)]
        expected_hash: String,
    },
}
