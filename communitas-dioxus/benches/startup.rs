// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Startup performance benchmarks for Communitas Dioxus application.
//!
//! Measures the time to bootstrap UI services, which is the primary
//! controllable portion of startup time. The full startup also includes
//! Tauri/WebView initialization which is platform-dependent.
//!
//! Run benchmarks: `cargo bench -p communitas-dioxus --bench startup`
//! Run as test: `cargo bench -p communitas-dioxus --bench startup -- --test`

use criterion::{Criterion, criterion_group, criterion_main};

/// Benchmark UiServices bootstrap - the main controllable startup cost.
///
/// This measures:
/// - Storage discovery
/// - Identity generation
/// - CommunitasApp initialization
/// - All service creation (auth, navigation, messaging, kanban, etc.)
fn bench_services_bootstrap(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    c.bench_function("services_bootstrap", |b| {
        b.to_async(&rt).iter(|| async {
            // Use bootstrap_async which is the recommended async initialization
            let services = communitas_ui_service::UiServices::bootstrap_async()
                .await
                .expect("Bootstrap failed");

            // Return services to prevent optimization from removing the call
            std::hint::black_box(services)
        });
    });
}

/// Benchmark storage discovery alone - useful for isolating I/O costs.
fn bench_storage_discovery(c: &mut Criterion) {
    c.bench_function("storage_discovery", |b| {
        b.iter(|| {
            let storage = communitas_ui_service::storage::UiStorage::discover()
                .expect("Storage discovery failed");
            std::hint::black_box(storage)
        });
    });
}

criterion_group!(
    name = startup_benches;
    config = Criterion::default()
        .sample_size(10)  // Fewer samples since bootstrap is slow
        .measurement_time(std::time::Duration::from_secs(30));
    targets = bench_services_bootstrap, bench_storage_discovery
);

criterion_main!(startup_benches);
