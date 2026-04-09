use crate::hash;
use crate::storage;
use std::fs;
use std::path::Path;

/// Compute True ID for a region.
/// True ID = xxh3_64(parent_region_hash + "_" + child_region_hash)
/// For root level: True ID = xxh3_64(file_hash + "_" + region_hash)
pub fn compute(file_path: &str, region_hash: &str, parent_true_id: Option<&str>) -> String {
    let file_path = Path::new(file_path);
    
    // Read and normalize file content
    let raw = match fs::read(file_path) {
        Ok(b) => b,
        Err(e) => panic!("IO_ERROR: cannot read file: {}", e),
    };
    
    let normalized = crate::matcher::normalize_line_endings(&raw);
    
    // Compute file hash
    let file_hash = hash::compute(&normalized);
    
    // Compute True ID based on level
    let true_id = if let Some(parent) = parent_true_id {
        // Child level: parent_region_hash + "_" + child_region_hash
        hash::compute(format!("{}_{}", parent, region_hash).as_bytes())
    } else {
        // Root level: file_hash + "_" + region_hash
        hash::compute(format!("{}_{}", file_hash, region_hash).as_bytes())
    };
    
    true_id
}

/// Save True ID metadata to storage
pub fn save_true_id(file_path: &str, region_hash: &str, parent_true_id: Option<&str>) -> Result<(), String> {
    let true_id = compute(file_path, region_hash, parent_true_id);
    let source_path = Path::new(file_path)
        .canonicalize()
        .map_err(|e| format!("IO_ERROR: cannot resolve path: {}", e))?
        .to_string_lossy()
        .to_string();
    
    let meta = storage::AnchorMeta {
        file: source_path,
        anchor: "".to_string(), // Not used for True ID storage
        hash: region_hash.to_string(),
        line_range: (0, 0), // Will be filled by caller
    };
    
    storage::save_anchor_metadata_with_true_id(&meta, &true_id, parent_true_id)
}
