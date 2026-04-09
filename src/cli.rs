use clap::{Parser, Subcommand};

// SPEC NOTE:
// Inline CLI arguments are assumed to be valid UTF-8 by the OS/CLI layer.
// AnchorScope enforces UTF-8 only for file-based inputs explicitly.

#[derive(Parser)]
#[command(name = "anchorscope", version = "1.2.0", about = "AnchorScope v1.2.0")]
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
        #[arg(long, conflicts_with_all = ["label"])]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long, conflicts_with_all = ["label"])]
        anchor_file: Option<String>,

        /// Expected xxh3 hash (hex) of the matched region.
        #[arg(long, conflicts_with = "label")]
        expected_hash: Option<String>,

        /// Use a human-readable label to identify the anchor.
        #[arg(long, conflicts_with_all = ["anchor", "anchor_file", "expected_hash"])]
        label: Option<String>,

        /// Replacement string (replaces the entire anchor region).
        #[arg(long)]
        replacement: String,
    },

    /// Assign a human-readable name to a True ID.
    Label {
        /// Human-readable name.
        #[arg(long)]
        name: String,

        /// True ID (hash from read output).
        #[arg(long)]
        true_id: String,
    },

    /// Display current buffer structure.
    Tree {
        /// Path to the target file.
        #[arg(long)]
        file: String,
    },
}
