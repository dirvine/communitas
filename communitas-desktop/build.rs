// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

fn main() {
    // Emit cargo:dev instruction expected by tauri-build 2.5.1
    println!("cargo:dev=false");
    
    // Always build Tauri for the desktop app
    tauri_build::build();
}
