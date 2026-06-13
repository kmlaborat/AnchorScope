use std::path::PathBuf;

/// Returns {TMPDIR}/anchorscope base path
pub fn anchorscope_temp_dir() -> PathBuf {
    std::env::temp_dir().join("anchorscope")
}

/// Returns {TMPDIR}/anchorscope/anchors for v1.1.0 compatibility
pub fn anchors_dir() -> PathBuf {
    anchorscope_temp_dir().join("anchors")
}

/// Returns {TMPDIR}/anchorscope/labels for alias storage
pub fn labels_dir() -> PathBuf {
    anchorscope_temp_dir().join("labels")
}

/// Returns {TMPDIR}/anchorscope/{file_hash}
pub fn file_dir(file_hash: &str) -> PathBuf {
    anchorscope_temp_dir().join(file_hash)
}

/// Returns {TMPDIR}/anchorscope/{file_hash}/{true_id}
pub fn true_id_dir(file_hash: &str, true_id: &str) -> PathBuf {
    file_dir(file_hash).join(true_id)
}

/// Returns {TMPDIR}/anchorscope/{file_hash}/{parent_true_id}/{true_id} for nested
pub fn nested_true_id_dir(file_hash: &str, parent_true_id: &str, true_id: &str) -> PathBuf {
    true_id_dir(file_hash, parent_true_id).join(true_id)
}
