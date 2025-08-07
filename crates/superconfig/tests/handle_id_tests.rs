//! Unit tests for handle ID generation and management

use serial_test::serial;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use superconfig::types::{
    HandleID, generate_handle_id, get_current_handle_count, reset_handle_counter,
};

#[test]
#[serial]
fn test_generate_unique_ids() {
    reset_handle_counter();

    let id1 = generate_handle_id();
    let id2 = generate_handle_id();
    let id3 = generate_handle_id();

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
#[serial]
fn test_thread_safety() {
    reset_handle_counter();

    let handles = Arc::new(std::sync::Mutex::new(HashSet::new()));
    let mut threads = vec![];

    // Spawn 10 threads, each generating 100 IDs
    for _ in 0..10 {
        let handles_clone = Arc::clone(&handles);
        let handle = thread::spawn(move || {
            let mut local_ids = Vec::new();
            for _ in 0..100 {
                local_ids.push(generate_handle_id());
            }

            let mut handles_set = handles_clone.lock().unwrap();
            for id in local_ids {
                assert!(handles_set.insert(id), "Duplicate ID generated: {}", id);
            }
        });
        threads.push(handle);
    }

    // Wait for all threads to complete
    for handle in threads {
        handle.join().unwrap();
    }

    // Should have exactly 1000 unique IDs
    let handles_set = handles.lock().unwrap();
    assert_eq!(handles_set.len(), 1000);
}

#[test]
#[serial]
fn test_reset_counter() {
    reset_handle_counter();
    assert_eq!(get_current_handle_count(), 1);

    let _ = generate_handle_id();
    let _ = generate_handle_id();
    assert_eq!(get_current_handle_count(), 3);

    reset_handle_counter();
    assert_eq!(get_current_handle_count(), 1);

    let id = generate_handle_id();
    assert_eq!(id, 1);
}

#[test]
#[serial]
fn test_get_current_handle_count() {
    reset_handle_counter();

    let initial = get_current_handle_count();
    assert_eq!(initial, 1);

    let _ = generate_handle_id();
    let after_one = get_current_handle_count();
    assert_eq!(after_one, 2);

    let _ = generate_handle_id();
    let _ = generate_handle_id();
    let after_three = get_current_handle_count();
    assert_eq!(after_three, 4);
}

#[test]
fn test_handle_id_type_compatibility() {
    // Ensure HandleID is u64 for FFI compatibility
    assert_eq!(std::mem::size_of::<HandleID>(), std::mem::size_of::<u64>());

    let id: HandleID = generate_handle_id();
    let as_u64: u64 = id;
    assert_eq!(id, as_u64);
}

#[test]
#[serial]
fn test_large_number_of_ids() {
    // Don't reset counter - just test uniqueness of generated IDs
    // This avoids race conditions with other tests

    // Generate many IDs to test for uniqueness
    let mut ids = HashSet::new();
    let mut prev_id = 0;

    for _ in 0..10000 {
        let id = generate_handle_id();
        // Each ID should be unique
        assert!(ids.insert(id), "Duplicate ID: {}", id);
        // IDs should be monotonically increasing
        assert!(
            id > prev_id,
            "ID {} not greater than previous {}",
            id,
            prev_id
        );
        prev_id = id;
    }

    assert_eq!(ids.len(), 10000);
}

#[test]
#[serial]
fn test_atomic_ordering() {
    reset_handle_counter();

    // Test that relaxed ordering works correctly
    let id1 = generate_handle_id();
    let current = get_current_handle_count();
    let id2 = generate_handle_id();

    assert_eq!(id1, 1);
    assert_eq!(current, 2);
    assert_eq!(id2, 2);
}
