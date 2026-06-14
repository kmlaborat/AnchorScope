use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "anchorscope", version = "2.0.0", about = "AnchorScope v2.0.0 — Deterministic scoped editing protocol")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Match anchor, compute and return scope_hash and matched content.
    Read {
        /// Path to the target file.
        #[arg(long)]
        file: String,

        /// Anchor string (exact, multi-line allowed via escape sequences).
        #[arg(long)]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long, conflicts_with = "anchor")]
        anchor_file: Option<String>,
    },

    /// Verify hash, apply replacement.
    Write {
        /// Path to the target file.
        #[arg(long)]
        file: String,

        /// Anchor string — must match exactly.
        #[arg(long)]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long, conflicts_with = "anchor")]
        anchor_file: Option<String>,

        /// Expected xxh3_64 hash (hex) of the matched scope.
        #[arg(long)]
        expected_hash: String,

        /// Replacement string (replaces the entire anchor scope).
        #[arg(long)]
        replacement: Option<String>,

        /// Path to a file containing the replacement content.
        #[arg(long, conflicts_with = "replacement")]
        replacement_file: Option<String>,
    },
}
