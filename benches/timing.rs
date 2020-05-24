use clocksource::Clocksource;
use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use quanta::Clock;
use std::time::Instant;

fn time_instant_now(b: &mut Bencher) {
    b.iter(|| Instant::now())
}

fn time_clocksource_time(b: &mut Bencher) {
    let cs = Clocksource::new();
    b.iter(|| cs.time())
}

fn time_clocksource_counter(b: &mut Bencher) {
    let cs = Clocksource::new();
    b.iter(|| cs.counter())
}

fn time_clocksource_counter_scaled(b: &mut Bencher) {
    let cs = Clocksource::new();
    b.iter(|| cs.convert(cs.counter()))
}

fn time_quanta_now(b: &mut Bencher) {
    let mut clock = Clock::new();
    b.iter(|| clock.now())
}

fn time_quanta_raw(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.raw())
}

fn time_quanta_raw_scaled(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.scaled(clock.raw()))
}

fn time_quanta_start(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.start())
}

fn time_quanta_start_scaled(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.scaled(clock.start()))
}

fn time_quanta_end(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.end())
}

fn time_quanta_end_scaled(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.scaled(clock.end()))
}

fn time_instant_delta(b: &mut Bencher) {
    b.iter(|| {
        let start = Instant::now();
        let d = Instant::now() - start;
        (d.as_secs() * 1_000_000_000) + u64::from(d.subsec_nanos())
    })
}

fn time_clocksource_counter_delta(b: &mut Bencher) {
    let cs = Clocksource::new();
    b.iter(|| {
        let start = cs.counter();
        let end = cs.counter();
        cs.convert(end) - cs.convert(start)
    })
}

fn time_clocksource_time_delta(b: &mut Bencher) {
    let cs = Clocksource::new();
    b.iter(|| {
        let t0 = cs.time();
        let t1 = cs.time();
        t1 - t0
    })
}

fn time_quanta_raw_delta(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| {
        let start = clock.raw();
        let end = clock.raw();
        clock.delta(start, end)
    })
}

fn time_quanta_now_delta(b: &mut Bencher) {
    let mut clock = Clock::new();
    b.iter(|| {
        let start = clock.now();
        let end = clock.now();
        end - start
    })
}

fn time_quanta_start_end_delta(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| {
        let start = clock.start();
        let end = clock.end();
        clock.delta(start, end)
    })
}

fn time_quanta_recent(b: &mut Bencher) {
    let clock = Clock::new();
    b.iter(|| clock.recent())
}

fn benchmark(c: &mut Criterion) {
    let mut std_group = c.benchmark_group("stdlib");
    std_group.bench_function("instant now", time_instant_now);
    std_group.bench_function("instant delta", time_instant_delta);

    std_group.finish();

    let mut cs_group = c.benchmark_group("clocksource");
    cs_group.bench_function("clocksource time", time_clocksource_time);
    cs_group.bench_function("clocksource time delta", time_clocksource_time_delta);
    cs_group.bench_function("clocksource counter", time_clocksource_counter);
    cs_group.bench_function(
        "clocksource counter scaled",
        time_clocksource_counter_scaled,
    );
    cs_group.bench_function("clocksource counter delta", time_clocksource_counter_delta);

    cs_group.finish();

    let mut q_group = c.benchmark_group("quanta");
    q_group.bench_function("quanta now", time_quanta_now);
    q_group.bench_function("quanta now delta", time_quanta_now_delta);
    q_group.bench_function("quanta raw", time_quanta_raw);
    q_group.bench_function("quanta raw scaled", time_quanta_raw_scaled);
    q_group.bench_function("quanta raw delta", time_quanta_raw_delta);
    q_group.bench_function("quanta start", time_quanta_start);
    q_group.bench_function("quanta start scaled", time_quanta_start_scaled);
    q_group.bench_function("quanta end", time_quanta_end);
    q_group.bench_function("quanta end scaled", time_quanta_end_scaled);
    q_group.bench_function("quanta start/end delta", time_quanta_start_end_delta);
    q_group.bench_function("quanta recent", time_quanta_recent);

    q_group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
