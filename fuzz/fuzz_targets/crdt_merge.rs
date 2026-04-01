// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_main]
use libfuzzer_sys::fuzz_target;
use yrs::{Doc, encode_state_v1, decode_state_v1};
use communitas_core::crdt::merge_docs;

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 { return; }
    let mut doc1 = Doc::new();
    let mut doc2 = Doc::new();
    // Apply random updates
    if let Ok(update) = decode_state_v1(data) {
        doc1.apply_update(update.clone());
        doc2.apply_update(update);
    }
    // Merge
    merge_docs(&mut doc1, &doc2);
    // Verify no crash
    let _ = encode_state_v1(&doc1);
});