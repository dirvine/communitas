# PLAN-27: Phase 2.1 — DriveService Real Implementation

**Milestone**: M3.1 Remediation
**Phase**: 2.1 (DriveService Parity)
**Status**: Pending
**Created**: 2026-01-20
**Depends on**: PLAN-26 (Phase 1 Complete)

---

## Overview

Replace the mock logic in `communitas-ui-service/src/drive.rs` with real calls into `communitas-core` Commands and Queries. The Command/Query API already exists (WriteFile, ReadFile, ListFiles, etc.) - we just need to wire the DriveService methods to use CommunitasApp.

## Prerequisites

- [x] PLAN-26 complete (MessagingService fully wired)
- [x] CommunitasApp available in UiServices
- [x] Virtual Disk Commands/Queries exist in communitas-core

---

## Tasks

<task type="auto" priority="p1">
  <n>Add CommunitasApp to DriveService constructor</n>
  <files>
    communitas-ui-service/src/drive.rs,
    communitas-ui-service/src/lib.rs
  </files>
  <action>
    1. Add `app: Arc<CommunitasApp>` field to DriveService struct
    2. Update DriveService::new() to accept `app` parameter
    3. Update UiServices::new() in lib.rs to pass app to DriveService
    4. Add `pub fn app(&self) -> Arc<CommunitasApp>` accessor
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
  </verify>
  <done>
    - DriveService holds Arc<CommunitasApp>
    - UiServices wires app to DriveService
    - Compiles without errors
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement list_disks with real Query::ListDisks</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock list_disks() implementation
    2. Call app.query(Query::ListDisks { entity_id })
    3. Convert QueryResponse::DiskList to UI DiskInfo type
    4. Handle errors appropriately (map QueryError to DriveError)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - list_disks returns real disk info from CommunitasApp
    - Error handling maps core errors to DriveError
    - Unit tests pass
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement list_directory with real Query::ListFiles</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock list_directory() implementation
    2. Build Query::ListFiles { entity_id, disk_type, path }
    3. Convert DiskTypeArg for query (Private/Public/Shared)
    4. Convert QueryResponse::FileList to UI FileInfo type
    5. Handle empty directories and non-existent paths
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - list_directory returns real file listing
    - Handles path navigation correctly
    - Error handling works for invalid paths
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement read_file with real Query::ReadFile</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock read_file() implementation
    2. Build Query::ReadFile { entity_id, disk_type, path }
    3. Convert QueryResponse::FileContents to Vec<u8>
    4. Handle file not found, permission errors
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - read_file returns actual file contents
    - FileNotFound errors handled
    - Large files work correctly
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement write_file with real Command::WriteFile</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock write_file() implementation
    2. Build Command::WriteFile { entity_id, disk_type, path, data }
    3. Execute command via app.execute()
    4. Handle FileWritten event from response
    5. Update watch channel with new file info
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - write_file persists data through CommunitasApp
    - Watch channel notified of new/updated files
    - Error handling for disk full, permission denied
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement delete_file with real Command::DeleteFile</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock delete_file() implementation
    2. Build Command::DeleteFile { entity_id, disk_type, path }
    3. Execute command via app.execute()
    4. Handle FileDeleted event
    5. Update watch channel
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - delete_file removes files through CommunitasApp
    - Watch channel updated
    - Non-existent file error handled
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement create_directory with real Command::CreateDirectory</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock create_directory() implementation
    2. Build Command::CreateDirectory { entity_id, disk_type, path }
    3. Execute and handle DirectoryCreated event
    4. Update watch channel
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - create_directory creates real directories
    - Nested directory creation works
    - Watch channel updated
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement move_file and copy_file with real Commands</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock move_file() with Command::MoveFile
    2. Replace mock copy_file() with Command::CopyFile
    3. Handle FileMoved/FileCopied events
    4. Update watch channel for both source and destination
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - move_file renames/moves through CommunitasApp
    - copy_file duplicates through CommunitasApp
    - Watch channel reflects changes
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement get_disk_stats and get_file_preview</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Replace mock get_disk_stats() with Query::GetDiskStats
    2. Replace mock get_file_preview() with Query::GetFilePreview
    3. Convert response types appropriately
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Disk stats show real usage
    - File previews generated for supported types
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire upload/download progress tracking</n>
  <files>
    communitas-ui-service/src/drive.rs
  </files>
  <action>
    1. Update start_upload() to use real write operations
    2. Update start_download() to use real read operations
    3. Maintain progress tracking via watch channels
    4. Support checksum verification on completion
    5. Handle cancellation properly
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Uploads write real data with progress
    - Downloads read real data with progress
    - Checksums verified
    - Cancellation stops operation
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add integration tests for DriveService</n>
  <files>
    communitas-ui-service/tests/drive_integration.rs
  </files>
  <action>
    1. Create integration test file
    2. Test full file lifecycle: write -> read -> move -> delete
    3. Test directory operations
    4. Test disk stats accuracy
    5. Test error cases
  </action>
  <verify>
    cargo test -p communitas-ui-service --test drive_integration
  </verify>
  <done>
    - Integration tests pass with real CommunitasApp
    - All CRUD operations verified
    - Error handling verified
  </done>
</task>

---

## Exit Criteria

- [ ] All DriveService methods use real CommunitasApp calls
- [ ] No mock data returned
- [ ] Integration tests pass
- [ ] Watch channels update reactively
- [ ] Upload/download progress tracking works

---

## Next

PLAN-28: CallService Integration (Wire to saorsa-webrtc-*)
