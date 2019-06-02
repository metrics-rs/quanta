#![cfg_attr(test, feature(test))]
mod bench {
    extern crate clocksource;
    extern crate test;
    use self::test::Bencher;
    use clocksource::Clocksource;
    use quanta::Clock;
    use std::time::Instant;

    #[bench]
    fn time_instant_now(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(Instant::now());
            }
        });
    }

    #[bench]
    fn time_instant_delta(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
                let start = Instant::now();
                let d = Instant::now() - start;
                test::black_box((d.as_secs() * 1_000_000_000) + u64::from(d.subsec_nanos()));
            }
        });
    }

    #[bench]
    fn time_clocksource_counter(b: &mut Bencher) {
        let cs = Clocksource::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.counter());
            }
        });
    }

    #[bench]
    fn time_clocksource_counter_delta(b: &mut Bencher) {
        let cs = Clocksource::new();
        b.iter(|| {
            for _ in 0..1000 {
                let start = cs.counter();
                let end = cs.counter();
                let t0 = cs.convert(start);
                let t1 = cs.convert(end);
                test::black_box(t1 - t0);
            }
        });
    }

    #[bench]
    fn time_clocksource_time(b: &mut Bencher) {
        let cs = Clocksource::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.time());
            }
        });
    }

    #[bench]
    fn time_clocksource_time_delta(b: &mut Bencher) {
        let cs = Clocksource::new();
        b.iter(|| {
            for _ in 0..1000 {
                let t0 = cs.time();
                let t1 = cs.time();
                test::black_box(t1 - t0);
            }
        });
    }

    #[bench]
    fn time_quanta_now(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.now());
            }
        });
    }

    #[bench]
    fn time_quanta_now_delta(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                let start = cs.now();
                let end = cs.now();
                test::black_box(end - start);
            }
        });
    }

    #[bench]
    fn time_quanta_start(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.start());
            }
        });
    }

    #[bench]
    fn time_quanta_end(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.end());
            }
        });
    }

    #[bench]
    fn time_quanta_start_end_delta(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                let start = cs.start();
                let end = cs.end();
                test::black_box(cs.delta(start, end));
            }
        });
    }

    #[bench]
    fn time_quanta_raw(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.raw());
            }
        });
    }

    #[bench]
    fn time_quanta_raw_delta(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                let start = cs.raw();
                let end = cs.raw();
                test::black_box(cs.delta(start, end));
            }
        });
    }

    #[bench]
    fn time_quanta_recent(b: &mut Bencher) {
        let cs: Clock = Clock::new();
        b.iter(|| {
            for _ in 0..1000 {
                test::black_box(cs.recent());
            }
        });
    }
}
