use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lox_bytecode_vm::core::value::Object;
use lox_bytecode_vm::core::value::Value;

// Configure criterion for more consistent benchmarks
fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100) // Increase sample size from default
        .measurement_time(std::time::Duration::from_secs(2))
        .nresamples(10_000) // More resampling for better statistical analysis
        .noise_threshold(0.05) // Lower threshold to detect smaller differences
}

fn creation_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Value Creation");

    // Benchmark number creation
    group.bench_function("number", |b| b.iter(|| Value::number(black_box(42.0))));

    // Benchmark boolean creation
    group.bench_function("boolean", |b| b.iter(|| Value::boolean(black_box(true))));

    // Benchmark nil creation
    group.bench_function("nil", |b| b.iter(|| Value::nil()));

    // Note: In this implementation, object references are stored,
    // not the actual objects. The VM would handle object allocation.
    // We'll benchmark creating object references.
    group.bench_function("object_reference", |b| {
        b.iter(|| Value::object(black_box(42)))
    });

    group.finish();
}

fn cloning_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Value Cloning");

    // Benchmark number cloning
    let num = Value::number(42.0);
    group.bench_function("number", |b| b.iter(|| black_box(num)));

    // Benchmark boolean cloning
    let bool_val = Value::boolean(true);
    group.bench_function("boolean", |b| b.iter(|| black_box(bool_val)));

    // Benchmark nil cloning
    let nil_val = Value::nil();
    group.bench_function("nil", |b| b.iter(|| black_box(nil_val)));

    // Benchmark object reference cloning
    let obj_ref = Value::object(123);
    group.bench_function("object_reference", |b| b.iter(|| black_box(obj_ref)));

    group.finish();
}

fn type_check_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Type Checking");

    // Values of different types
    let num = Value::number(42.0);
    let bool_val = Value::boolean(true);
    let nil_val = Value::nil();
    let obj_ref = Value::object(123);

    // Benchmark is_number
    group.bench_function("is_number", |b| b.iter(|| black_box(num.is_number())));

    // Benchmark is_boolean
    group.bench_function("is_boolean", |b| {
        b.iter(|| black_box(bool_val.is_boolean()))
    });

    // Benchmark is_nil
    group.bench_function("is_nil", |b| b.iter(|| black_box(nil_val.is_nil())));

    // Benchmark is_object
    group.bench_function("is_object", |b| b.iter(|| black_box(obj_ref.is_object())));

    group.finish();
}

fn value_access_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Value Access");

    // Benchmark accessing number
    let num = Value::number(42.0);
    group.bench_function("as_number", |b| b.iter(|| black_box(num.as_number())));

    // Benchmark accessing boolean
    let bool_val = Value::boolean(true);
    group.bench_function("as_boolean", |b| {
        b.iter(|| black_box(bool_val.as_boolean()))
    });

    // Benchmark accessing object
    let obj_ref = Value::object(123);
    group.bench_function("as_object", |b| b.iter(|| black_box(obj_ref.as_object())));

    group.finish();
}

fn stack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Stack Operations");

    // Simulate VM stack operations pushing/popping primitives
    let mut stack = Vec::with_capacity(100);
    group.bench_function("push_pop_100_numbers", |b| {
        b.iter(|| {
            stack.clear();
            for i in 0..100 {
                stack.push(Value::number(i as f64));
            }
            while let Some(_) = stack.pop() {}
        })
    });

    // Simulate VM stack operations with object references
    group.bench_function("push_pop_100_object_refs", |b| {
        b.iter(|| {
            stack.clear();
            for i in 0..100 {
                stack.push(Value::object(i));
            }
            while let Some(_) = stack.pop() {}
        })
    });

    // Mix of different value types
    group.bench_function("push_pop_mixed_values", |b| {
        b.iter(|| {
            stack.clear();
            for i in 0..100 {
                match i % 4 {
                    0 => stack.push(Value::number(i as f64)),
                    1 => stack.push(Value::boolean(i % 2 == 0)),
                    2 => stack.push(Value::object(i)),
                    _ => stack.push(Value::nil()),
                }
            }
            while let Some(_) = stack.pop() {}
        })
    });

    group.finish();
}

fn arithmetic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Arithmetic Operations");

    // Setup values for arithmetic
    let n1 = Value::number(3.14);
    let n2 = Value::number(2.71);

    // Benchmark addition
    group.bench_function("add", |b| {
        b.iter(|| {
            let result = n1.as_number() + n2.as_number();
            black_box(Value::number(result))
        })
    });

    // Benchmark subtraction
    group.bench_function("subtract", |b| {
        b.iter(|| {
            let result = n1.as_number() - n2.as_number();
            black_box(Value::number(result))
        })
    });

    // Benchmark multiplication
    group.bench_function("multiply", |b| {
        b.iter(|| {
            let result = n1.as_number() * n2.as_number();
            black_box(Value::number(result))
        })
    });

    // Benchmark division
    group.bench_function("divide", |b| {
        b.iter(|| {
            let result = n1.as_number() / n2.as_number();
            black_box(Value::number(result))
        })
    });

    group.finish();
}

// Compare NaN boxing with other value representations
fn compare_with_enum_style(c: &mut Criterion) {
    // This would benchmark the NaN-boxed value against a traditional enum-style value
    // For a real comparison, you'd need to include the previous Value implementation
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = creation_benchmarks, cloning_benchmarks, type_check_benchmarks,
              value_access_benchmarks, stack_operations, arithmetic_operations
}
criterion_main!(benches);
