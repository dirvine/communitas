// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

fn main() {
    // WORKAROUND: tauri-build 2.5.1 expects DEP_TAURI_DEV env var
    // This should be set by tauri's build script, but it doesn't propagate
    // to build dependencies. Setting directly is safe in build.rs single-threaded context.
    // Remove this when tauri-build 2.9+ is published to crates.io.
    // Note: unsafe required but this is safe - build.rs is single-threaded
    unsafe {
        std::env::set_var("DEP_TAURI_DEV", "false");
    }

    tauri_build::build();
}
