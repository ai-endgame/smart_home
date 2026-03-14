use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use smart_home::{
    automation::{Action, AutomationEngine, Trigger},
    manager::SmartHome,
    models::{DeviceState, DeviceType},
};

// ── Helpers ──────────────────────────────────────────────────────────

fn make_home_with_devices(n: usize) -> SmartHome {
    let mut home = SmartHome::new();
    for i in 0..n {
        home.add_device(&format!("device_{i}"), DeviceType::Light)
            .unwrap();
    }
    home
}

fn make_home_with_room(n_devices: usize) -> SmartHome {
    let mut home = SmartHome::new();
    home.add_room("living room").unwrap();
    for i in 0..n_devices {
        let name = format!("device_{i}");
        home.add_device(&name, DeviceType::Light).unwrap();
        home.assign_device_to_room(&name, "living room").unwrap();
    }
    home
}

fn make_engine_with_rules(n: usize) -> AutomationEngine {
    let mut engine = AutomationEngine::new();
    for i in 0..n {
        engine
            .add_rule(
                &format!("rule_{i}"),
                Trigger::DeviceStateChange {
                    device_name: format!("device_{i}"),
                    target_state: DeviceState::On,
                },
                Action::DeviceState {
                    device_name: format!("device_{i}"),
                    state: DeviceState::Off,
                },
            )
            .unwrap();
    }
    engine
}

// ── Benchmarks ───────────────────────────────────────────────────────

/// get_room_devices: the hot path that does O(n*m) linear scans.
fn bench_get_room_devices(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_room_devices");
    for size in [10usize, 100, 500, 1000] {
        let home = make_home_with_room(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(home.get_room_devices("living room")));
        });
    }
    group.finish();
}

/// list_devices: sorts on every call.
fn bench_list_devices(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_devices");
    for size in [10usize, 100, 500, 1000] {
        let home = make_home_with_devices(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(home.list_devices()));
        });
    }
    group.finish();
}

/// add_device: UUID generation + HashMap insert.
fn bench_add_device(c: &mut Criterion) {
    c.bench_function("add_device", |b| {
        b.iter_with_setup(
            SmartHome::new,
            |mut home| {
                // Use a static name since we measure one insert at a time
                let _ = home.add_device(black_box("my lamp"), DeviceType::Light);
            },
        );
    });
}

/// get_device: HashMap lookup with lowercasing.
fn bench_get_device(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_device");
    for size in [10usize, 100, 1000] {
        let home = make_home_with_devices(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(home.get_device("device_0")));
        });
    }
    group.finish();
}

/// evaluate_rules: iterates all rules, calls get_device per rule.
fn bench_evaluate_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate_rules");
    for size in [10usize, 100, 500, 1000] {
        let home = make_home_with_devices(size);
        let engine = make_engine_with_rules(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(engine.evaluate_rules(&home)));
        });
    }
    group.finish();
}

/// add_rule: Vec linear scan for duplicate check.
fn bench_add_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_rule");
    for size in [10usize, 100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &sz| {
            b.iter_with_setup(
                || make_engine_with_rules(sz),
                |mut engine| {
                    // Adding a new rule at the end — must scan all existing rules first
                    let _ = engine.add_rule(
                        black_box("new_rule"),
                        Trigger::DeviceStateChange {
                            device_name: "x".to_string(),
                            target_state: DeviceState::On,
                        },
                        Action::DeviceState {
                            device_name: "y".to_string(),
                            state: DeviceState::Off,
                        },
                    );
                },
            );
        });
    }
    group.finish();
}

/// remove_rule: Vec linear scan + O(n) shift.
fn bench_remove_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_rule");
    for size in [10usize, 100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &sz| {
            b.iter_with_setup(
                || make_engine_with_rules(sz),
                |mut engine| {
                    // Remove from the middle to stress the O(n) shift
                    let mid = sz / 2;
                    let _ = engine.remove_rule(&format!("rule_{mid}"));
                },
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_get_room_devices,
    bench_list_devices,
    bench_add_device,
    bench_get_device,
    bench_evaluate_rules,
    bench_add_rule,
    bench_remove_rule,
);
criterion_main!(benches);
