//! Core flow benchmarks for messaging and drive services.

use std::sync::Arc;

use communitas_bench::communitas_core::app::CommunitasApp;
use communitas_bench::communitas_ui_api::drive::DiskType;
use communitas_bench::communitas_ui_service::auth::AuthController;
use communitas_bench::communitas_ui_service::drive::DriveService;
use communitas_bench::communitas_ui_service::messaging::MessagingService;
use communitas_bench::communitas_ui_service::storage::UiStorage;
use communitas_bench::tempfile::TempDir;
use communitas_bench::tokio;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Create a temporary directory for benchmark isolation.
fn create_temp_dir() -> TempDir {
    TempDir::new().expect("bench: create temp dir")
}

/// Create an authenticated messaging service for benchmarks.
async fn create_messaging_service(temp: &TempDir) -> MessagingService {
    let storage = UiStorage::from_path(temp.path()).expect("bench: create storage");
    let auth = Arc::new(AuthController::new(storage).expect("bench: create auth"));
    auth.enable_demo_mode();
    let app: Arc<CommunitasApp> = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "BenchUser".to_string(),
            "BenchDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .expect("bench: create app"),
    );
    MessagingService::new(auth, app)
}

/// Create an authenticated drive service for benchmarks.
async fn create_drive_service(temp: &TempDir) -> DriveService {
    let storage = UiStorage::from_path(temp.path()).expect("bench: create storage");
    let auth = Arc::new(AuthController::new(storage).expect("bench: create auth"));
    auth.enable_demo_mode();
    let app: Arc<CommunitasApp> = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "BenchUser".to_string(),
            "BenchDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .expect("bench: create app"),
    );
    DriveService::new(auth, app)
}

fn auth_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("auth");

    // Placeholder - real benchmarks will mock the auth flow
    group.bench_function("placeholder", |b| b.iter(|| black_box(1 + 1)));

    group.finish();
}

fn messaging_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: create runtime");
    let mut group = c.benchmark_group("messaging");

    // Benchmark list_threads
    group.bench_function("list_threads", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_messaging_service(&temp));
        b.to_async(&rt).iter(|| async {
            let result = service.list_threads().await;
            black_box(result)
        });
    });

    // Benchmark get_messages with default pagination
    group.bench_function("get_messages", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_messaging_service(&temp));
        b.to_async(&rt).iter(|| async {
            // Use a non-existent thread to benchmark the lookup path
            let result = service
                .get_messages(black_box("thread-bench"), black_box(50), None)
                .await;
            black_box(result)
        });
    });

    // Benchmark send_message
    group.bench_function("send_message", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_messaging_service(&temp));
        b.to_async(&rt).iter(|| async {
            let result = service
                .send_message(
                    black_box("thread-bench"),
                    black_box("Benchmark message content"),
                    None, // no reply
                )
                .await;
            black_box(result)
        });
    });

    group.finish();
}

fn kanban_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("kanban");

    // Placeholder for kanban benchmarks
    group.bench_function("placeholder", |b| b.iter(|| black_box(3 + 3)));

    group.finish();
}

fn drive_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: create runtime");
    let mut group = c.benchmark_group("drive");

    // Benchmark list_directory
    group.bench_function("list_directory", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_drive_service(&temp));
        b.to_async(&rt).iter(|| async {
            let result = service
                .list_directory(
                    black_box("entity-bench"),
                    black_box(DiskType::Private),
                    black_box("/"),
                )
                .await;
            black_box(result)
        });
    });

    // Benchmark read_file
    group.bench_function("read_file", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_drive_service(&temp));
        b.to_async(&rt).iter(|| async {
            let result = service
                .read_file(
                    black_box("entity-bench"),
                    black_box(DiskType::Private),
                    black_box("/test.txt"),
                )
                .await;
            black_box(result)
        });
    });

    // Benchmark write_file
    group.bench_function("write_file", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_drive_service(&temp));
        let content: &[u8] = b"Benchmark file content for performance testing.";
        b.to_async(&rt).iter(|| async {
            let result = service
                .write_file(
                    black_box("entity-bench"),
                    black_box(DiskType::Private),
                    black_box("/bench_file.txt"),
                    black_box(content),
                )
                .await;
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    auth_benchmarks,
    messaging_benchmarks,
    kanban_benchmarks,
    drive_benchmarks
);
criterion_main!(benches);
