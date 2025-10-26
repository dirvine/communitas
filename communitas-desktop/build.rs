// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

fn main() {
    //  Workaround for tauri-build 2.5.1 expecting DEP_TAURI_DEV
    // The tauri crate should emit this, but it doesn't propagate to build dependencies
    // See: https://github.com/tauri-apps/tauri/issues/10591
    unsafe {
        std::env::set_var("DEP_TAURI_DEV", "false");
    }
    
    // Build Tauri for the desktop app
    tauri_build::build();
}
