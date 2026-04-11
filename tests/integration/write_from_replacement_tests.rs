mod write_from_replacement_tests {
    use serial_test::serial;
    use crate::storage;
    use crate::buffer_path;
    use crate::commands::write;
    use crate::hash;

    #[test]
    #[serial]
    fn test_write_from_replacement_uses_buffer_content() {
        // Setup: Create buffer with replacement file
        let content = b"def foo():\n    pass";
        let file_hash = hash::compute(content);
        let true_id = "test_write_from_replacement";
        let source_path = std::env::temp_dir().join("test_from_replacement.py");

        // Save file content and source path
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, source_path.to_str().unwrap()).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(&file_hash, &true_id, &storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            region_hash: hash::compute(content),
            anchor: "def foo()".to_string(),
        }).unwrap();

        // Create replacement file (simulating pipe output)
        let replacement = b"def foo():\n    return 42\n";
        let replacement_path = buffer_path::true_id_dir(&file_hash, &true_id).join("replacement");
        std::fs::write(&replacement_path, replacement).unwrap();

        // Save a label for this true_id
        storage::save_label_mapping("my_function", &true_id).unwrap();

        // Write using label (which will use --from-replacement via buffer)
        let exit_code = write::execute(
            source_path.to_str().unwrap(),
            None,
            None,
            None,
            Some("my_function"),
            "",  // replacement ignored when from_replacement is true
            true,  // from_replacement = true
        );

        assert_eq!(exit_code, 0, "write should succeed with --from-replacement");

        // Verify file was replaced with replacement content
        let final_content = std::fs::read_to_string(&source_path).unwrap();
        assert_eq!(final_content, "def foo():\n    return 42\n");

        // Cleanup
        storage::invalidate_label("my_function");
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_file(&source_path);
    }

    #[test]
    #[serial]
    fn test_write_from_replacement_fails_without_label() {
        // Setup
        let content = b"test content";
        let file_hash = hash::compute(content);
        let source_path = std::env::temp_dir().join("test_no_label.txt");

        // Save file content
        std::fs::write(&source_path, content).unwrap();
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, source_path.to_str().unwrap()).unwrap();

        // Try to use --from-replacement without label (should fail)
        let exit_code = write::execute(
            source_path.to_str().unwrap(),
            Some("test"),
            None,
            Some(&hash::compute(content)),
            None,
            "",
            true,  // from_replacement = true
        );

        assert_eq!(exit_code, 1);
        // Should output error message about not being able to use --from-replacement without --label

        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &file_hash).unwrap();
        let _ = std::fs::remove_file(&source_path);
    }

    #[test]
    #[serial]
    fn test_write_replacement_conflict_returns_ambiguous_replacement() {
        // Setup
        let content = b"test content";
        let file_hash = hash::compute(content);
        let true_id = "test_ambiguous";
        let source_path = std::env::temp_dir().join("test_ambiguous.txt");

        // Save file content
        std::fs::write(&source_path, content).unwrap();
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, source_path.to_str().unwrap()).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(&file_hash, &true_id, &storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            region_hash: hash::compute(content),
            anchor: "test".to_string(),
        }).unwrap();

        // Save a label
        storage::save_label_mapping("test_label", &true_id).unwrap();

        // Try to use both --replacement and --from-replacement (should fail)
        let exit_code = write::execute(
            source_path.to_str().unwrap(),
            None,
            None,
            None,
            Some("test_label"),
            "CONFLICT",  // replacement provided
            true,  // from_replacement also true
        );

        assert_eq!(exit_code, 1);

        // Cleanup
        storage::invalidate_label("test_label");
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_file(&source_path);
    }
}
