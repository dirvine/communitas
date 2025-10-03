use communitas_core::encrypted_storage::key_management::KeyManager;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;

fn benchmark_pbkdf2(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("pbkdf2");
    group.measurement_time(Duration::from_secs(30));

    for iterations in &[1_000u32, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            iterations,
            |b, &iterations| {
                b.to_async(&rt).iter(|| async move {
                    let key_manager = KeyManager::new(iterations, false).await.unwrap();
                    let password = black_box("test-password-12345");
                    let salt = black_box(vec![1u8; 32]);
                    key_manager.derive_key(password, &salt).await.unwrap()
                });
            },
        );
    }
    group.finish();
}

fn benchmark_password_hash(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("blake3_password_hash", |b| {
        b.to_async(&rt).iter(|| async {
            let key_manager = KeyManager::new(100_000, false).await.unwrap();
            let password = black_box("test-password-12345");
            key_manager.hash_password(password).await.unwrap()
        });
    });
}

criterion_group!(benches, benchmark_pbkdf2, benchmark_password_hash);
criterion_main!(benches);
