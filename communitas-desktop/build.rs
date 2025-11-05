// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

fn main() {
    // WORKAROUND: tauri-build 2.5.1 expects cargo:dev instruction
    // This should be set by tauri's build script, but it doesn't propagate
    // to build dependencies. Emit the instruction directly to avoid the check.
    // Note: tauri-build versions follow different numbering than main tauri crate.
    // Latest tauri-build is 2.5.1 (as of 2025-11-05), not synchronized with tauri 2.9.x
    println!("cargo:dev=false");

    tauri_build::build();
}
