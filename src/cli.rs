use clap::{Parser, Subcommand};

// SPEC NOTE:
// Inline CLI arguments are assumed to be valid UTF-8 by the OS/CLI layer.
// AnchorScope enforces UTF-8 only for file-based inputs explicitly.

#[derive(Parser)]
#[command(name = "anchorscope", version = "1.3.0", about = "AnchorScope v1.3.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Find anchor in file and return location + hash.
    Read {
        /// Path to the target file.
        #[arg(long, conflicts_with = "true_id")]
        file: Option<String>,

        /// Anchor string (exact, multi-line allowed via escape sequences).
        /// Pass the anchor as a raw argument; use $'...' in shell for \n.
        #[arg(long)]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long)]
        anchor_file: Option<String>,

        /// Use a human-readable label to identify the parent buffer anchor.
        #[arg(long, conflicts_with = "true_id")]
        label: Option<String>,

        /// True ID (hash from read output). If specified, reads from buffer instead of file.
        #[arg(long, conflicts_with_all = ["file", "anchor_file", "label"])]
        true_id: Option<String>,
    },

    /// Replace anchor scope if expected_hash matches.
    Write {
        /// Path to the target file.
        #[arg(long, conflicts_with = "true_id")]
        file: Option<String>,

        /// Anchor string — must match exactly.
        /// Required when using --file, optional when using --true-id
        #[arg(long, conflicts_with = "label")]
        anchor: Option<String>,

        /// Path to a file containing the anchor string.
        #[arg(long, conflicts_with = "label", conflicts_with_all = ["true_id"])]
        anchor_file: Option<String>,

        /// Expected xxh3 hash (hex) of the matched scope.
        /// Required when using --true-id for buffer-based writing.
        #[arg(long, conflicts_with = "label")]
        expected_hash: Option<String>,

        /// Use a human-readable label to identify the anchor.
        #[arg(long, conflicts_with = "true_id")]
        label: Option<String>,

        /// True ID (hash from read output). If specified, writes to buffer instead of file.
        /// Cannot be used with --file, --anchor-file, or --label
        /// --anchor is optional and can be used with --true-id
        #[arg(long, conflicts_with_all = ["file", "anchor_file", "label"])]
        true_id: Option<String>,

        /// Replacement string (replaces the entire anchor scope).
        #[arg(long, default_value = "")]
        replacement: String,

        /// Use buffer's replacement file as replacement content.
        /// Cannot be used with --replacement.
        #[arg(long, conflicts_with = "replacement")]
        from_replacement: bool,
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

    /// Bridge Anchor Buffer and external tools via stdout/stdin or file I/O.
    Pipe {
        /// Use a human-readable label to identify the anchor.
        #[arg(long, conflicts_with = "true_id")]
        label: Option<String>,

        /// True ID (hash from read output).
        #[arg(long, conflicts_with = "label")]
        true_id: Option<String>,

        /// Output content to stdout (default mode).
        #[arg(long, conflicts_with_all = ["file_io", "tool"])]
        out: bool,

        /// Read from stdin and write to replacement (default mode).
        #[arg(long, conflicts_with_all = ["file_io", "tool", "out"], alias = "in")]
        in_flag: bool,

        /// File I/O mode: pass content path to external tool.
        #[arg(long, conflicts_with_all = ["out", "in_flag"], requires = "tool")]
        file_io: bool,

        /// External tool command to execute in file-io mode.
        #[arg(long)]
        tool: Option<String>,

        /// Arguments to pass to the tool (space-separated).
        #[arg(long, requires = "tool", allow_hyphen_values = true)]
        tool_args: Option<String>,
    },

    /// Return file paths of content and replacement for a True ID or alias.
    Paths {
        /// Use a human-readable label to identify the anchor.
        #[arg(long, conflicts_with = "true_id")]
        label: Option<String>,

        /// True ID (hash from read output).
        #[arg(long, conflicts_with = "label")]
        true_id: Option<String>,
    },
}
