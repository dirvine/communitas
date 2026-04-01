// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzz target for CRDT synchronization in communitas-core
#![no_main]

use libfuzzer_sys::fuzz_target;
use yrs::{Doc, Map, MapRef, Update};
use communitas_core::crdt::CrdtManager; // Adjust import as needed

fuzz_target!(|data: &[u8]| {
    if let Ok(update) = Update::decode_v1(data) {
        let mut doc = Doc::new();
        let mut txn = doc.transact_mut();
        let map: MapRef = doc.get_or_insert_map("fuzz_map");
        
        // Apply update (may panic on invalid, which is fine for fuzzing)
        if let Err(e) = txn.apply_update(update) {
            // Log or ignore invalid updates
            return;
        }
        
        // Simulate merge/conflict
        let keys: Vec<_> = map.keys(&txn).collect();
        for key in keys {
            map.insert(&mut txn, key, "fuzz_value");
        }
    }
});
