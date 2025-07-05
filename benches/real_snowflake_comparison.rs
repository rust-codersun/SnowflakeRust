use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use snowflake_generator::Snowflake;

fn benchmark_snowflake_batch_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Snowflake Batch Generation");
    
    // 设置测试时间
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    
    // 测试不同批次大小的ID生成
    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("Snowflake", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut snowflake = Snowflake::new(1, 1);
                    
                    for _ in 0..size {
                        black_box(snowflake.next_id().unwrap());
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_single_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single ID Generation");
    
    // 设置测试时间
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(2));
    
    // 测试单个ID生成的性能
    group.bench_function("Snowflake_single", |b| {
        let mut snowflake = Snowflake::new(1, 1);
        b.iter(|| {
            black_box(snowflake.next_id().unwrap());
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_snowflake_batch_generation,
    benchmark_single_id_generation
);
criterion_main!(benches);
