use crate::storage;
use crate::buffer_path;

#[test]
fn ambiguous_anchor_error_detection() {
    // Use a deterministic file_hash value
    let file_hash = "testhash_ambiguous";
    let true_id = "dup_true_id";

    // Create two parent directories under the same file_hash
    let parent1 = buffer_path::true_id_dir(file_hash, "parent1");
    let parent2 = buffer_path::true_id_dir(file_hash, "parent2");
    std::fs::create_dir_all(&parent1).unwrap();
    std::fs::create_dir_all(&parent2).unwrap();

    // Create the true_id directory under each parent (simulating duplicate anchors)
    let dup1 = parent1.join(true_id);
    let dup2 = parent2.join(true_id);
    std::fs::create_dir_all(&dup1).unwrap();
    std::fs::create_dir_all(&dup2).unwrap();

    // The storage function should detect ambiguity
    let result = storage::find_true_id_dir(file_hash, true_id);
    match result {
        Err(storage::AmbiguousAnchorError { true_id: tid, locations }) => {
            assert_eq!(tid, true_id);
            assert_eq!(locations.len(), 2);
        }
        _ => panic!("Ambiguous anchor not detected"),
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(buffer_path::file_dir(file_hash));
}
