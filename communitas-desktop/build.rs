// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

fn main() {
    // WORKAROUND: tauri-build 2.5.1 expects DEP_TAURI_DEV env var
    // This should be set by tauri's build script, but it doesn't propagate
    // to build dependencies. Using cargo:rustc-env instead of unsafe std::env::set_var
    // to avoid potential issues in CI environments.
    // Note: tauri-build versions follow different numbering than main tauri crate.
    // Latest tauri-build is 2.5.1 (as of 2025-11-05), not synchronized with tauri 2.9.x
    println!("cargo:rustc-env=DEP_TAURI_DEV=false");

    tauri_build::build();
}
