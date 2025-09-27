# Offline Entity Sync Overview

This document describes how Communitas now handles entity creation, updates, and deletions while offline, and how those changes are synchronized once connectivity returns.

## Entity Registry

* All organizations, groups, channels, projects, and contacts are persisted locally by `EntityDirectoryContext`.
* Each record stores metadata:
  * `syncStatus`: `new`, `synced`, `dirty`, `deleted`, or `error`.
  * `lastSyncedAt`: last successful sync timestamp.
  * `syncError`: most recent sync error (if any).
* Entity state is kept in `localStorage` for quick reloads (JSON serialized) and revived into `Date` objects on load.

## Operation Queue

* Every create/delete request enqueues an `EntitySyncOperation` containing:
  * `entityType` – organization, group, channel, project, or contact.
  * `operation` – `create`, `delete`, `update`, or `resolve`.
  * `payload` – serialized entity data (for creates) or `{ id }` for deletes.
  * `status` – `pending`, `processing`, or `failed`.
  * `attempts` – retry counter (max 3).
* Operations remain in the queue until the sync worker confirms completion.
* Failed operations are retried automatically (up to 3 attempts) whenever connectivity is available.

## Sync Worker

* A lightweight effect inside `EntityDirectoryContext` watches the operation queue.
* When the browser reports `navigator.onLine === true`, the first pending operation is marked `processing` and executed.
* Currently, network calls are placeholders (`await new Promise(resolve => setTimeout(resolve, 50))`). Hook in real API calls where indicated.
* Upon success:
  * `create` → entity `syncStatus` becomes `synced` and timestamps update.
  * `delete` → the entity is purged from local state.
* Upon failure:
  * Operation `status` reverts to `pending` if `attempts < 3`, otherwise it stays `failed` and surfaces the error.

## Offline Behavior

* Creating or removing entities while offline simply updates local state and enqueues the corresponding operation.
* Deleted entities stay visible (with `syncStatus: 'deleted'`) until the delete operation succeeds, unless they were never synced in which case they disappear immediately.
* Contacts and groups added offline can be accessed immediately in the UI; chat and storage panes work against the local registry.

## Online Reconciliation

* When connectivity returns, operations replay in insertion order.
* To integrate with a real backend:
  * Replace the `handleOperation` switch cases with real API calls.
  * Use the optional `newIdMap` parameter in `markOperationComplete` to swap temporary IDs with authoritative IDs.
  * Merge remote authoritative entity lists (pull from server) and call `setEntityStatus` / `purgeEntity` as needed for conflicts.

## Related APIs

* `useEntityDirectory()` now exposes:
  * `operations` – the queued sync jobs.
  * `enqueueOperation`, `markOperationComplete`, `markOperationFailed`, and `setEntityStatus` for advanced workflows.
* Consumers like the navigation sidebar and selector dialog now read from this context instead of static mock data.

## TODO / Future Enhancements

1. Replace placeholder sync logic with calls into Saorsa/Communitas backend endpoints.
2. Surface sync progress in the UI (e.g., badges on offline entities).
3. Add conflict resolution UI for simultaneous offline/online edits.
4. Persist the queue in IndexedDB for resilience after hard refreshes.
5. Synchronize chat/storage payloads via the same queue for consistent offline semantics.
