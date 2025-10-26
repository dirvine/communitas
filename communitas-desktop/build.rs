// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

fn main() {
    // Build Tauri for the desktop app
    // Note: tauri 2.8.x emits the cargo:dev instruction that tauri-build 2.5.1 expects
    tauri_build::build();
}
