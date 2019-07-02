# quanta

[![conduct-badge][]][conduct] [![travis-badge][]][travis] [![downloads-badge][] ![release-badge][]][crate] [![docs-badge][]][docs] [![libraries-io-badge][]][libraries-io] [![cargo-make-badge][]][cargo-make] [![license-badge][]](#license)

[conduct-badge]: https://img.shields.io/badge/%E2%9D%A4-code%20of%20conduct-blue.svg
[travis-badge]: https://img.shields.io/travis/metrics-rs/quanta/master.svg
[downloads-badge]: https://img.shields.io/crates/d/quanta.svg
[release-badge]: https://img.shields.io/crates/v/quanta.svg
[license-badge]: https://img.shields.io/crates/l/quanta.svg
[docs-badge]: https://docs.rs/quanta/badge.svg
[cargo-make-badge]: https://img.shields.io/badge/built%20with-cargo--make-yellow.svg
[cargo-make]: https://sagiegurari.github.io/cargo-make/
[libraries-io-badge]: https://img.shields.io/librariesio/github/metrics-rs/quanta.svg
[libraries-io]: https://libraries.io/cargo/quanta
[conduct]: https://github.com/metrics-rs/quanta/blob/master/CODE_OF_CONDUCT.md
[travis]: https://travis-ci.org/metrics-rs/quanta
[crate]: https://crates.io/crates/quanta
[docs]: https://docs.rs/quanta

__quanta__ is a high-speed timing library, useful for getting the current time _very quickly_.

## code of conduct

**NOTE**: All conversations and contributions to this project shall adhere to the [Code of Conduct][conduct].

## usage

The API documentation of this library can be found at [docs.rs/quanta](https://docs.rs/quanta/).

## general features
- time in nanoseconds
- super fast! (see the benchmarks)
- high-precision mode!
- mockable!
- cross-platform! (we target Linux, Windows, macOS, Solaris, \*BSD)
- fun, science-y name!

## performance

quanta provides high-speed access to the native system timing facilities and in general, with optimized assembly turned off, is generally on par with the standard library and external crates:

    test bench::time_clocksource_counter       ... bench:      30,060 ns/iter (+/- 2,051)
    test bench::time_clocksource_counter_delta ... bench:      74,790 ns/iter (+/- 2,897)
    test bench::time_clocksource_time          ... bench:      30,439 ns/iter (+/- 2,571)
    test bench::time_clocksource_time_delta    ... bench:      60,429 ns/iter (+/- 5,393)
    test bench::time_hotmic_now                ... bench:      30,202 ns/iter (+/- 1,643)
    test bench::time_hotmic_now_delta          ... bench:      59,499 ns/iter (+/- 5,829)
    test bench::time_hotmic_raw                ... bench:      29,371 ns/iter (+/- 2,110)
    test bench::time_hotmic_raw_delta          ... bench:      66,385 ns/iter (+/- 2,904)
    test bench::time_instant_delta             ... bench:      64,285 ns/iter (+/- 3,311)
    test bench::time_instant_now               ... bench:      18,603 ns/iter (+/- 1,116)

The non-delta tests represent the time it takes to take a single time measurement, while the delta tests represent the time to take two measurements and calculate the delta.  We can see that without using the optimized assembly features that both `quanta` and `clocksource` provide, taking single measurements is slower than [`Instant::now`] but generally consumes the same amount of time overall to take the measurements and calculate the delta, around 60-65ns.

Using optimized assembly, things can be much faster:

    test bench::time_clocksource_counter       ... bench:      11,424 ns/iter (+/- 848)
    test bench::time_clocksource_counter_delta ... bench:      36,813 ns/iter (+/- 2,047)
    test bench::time_clocksource_time          ... bench:      25,499 ns/iter (+/- 2,101)
    test bench::time_clocksource_time_delta    ... bench:      50,761 ns/iter (+/- 3,114)
    test bench::time_hotmic_now                ... bench:      18,918 ns/iter (+/- 1,591)
    test bench::time_hotmic_now_delta          ... bench:      38,367 ns/iter (+/- 2,134)
    test bench::time_hotmic_raw                ... bench:      10,984 ns/iter (+/- 814)
    test bench::time_hotmic_raw_delta          ... bench:      29,635 ns/iter (+/- 1,685)
    test bench::time_instant_delta             ... bench:      63,968 ns/iter (+/- 3,805)
    test bench::time_instant_now               ... bench:      18,096 ns/iter (+/- 1,381)

Both `quanta` and `clocksource` provide a way for the caller to get the "raw" measurement from the underlying time source, which is an unrefined value that needs to be scaled by a reference time source to end up as a meanginful value.  This is provided for taking measurments in tight loops where the deltas can be calculated after the fact.  For `clocksource`, the `counter` mode is the raw value, and `time` mode is the `Instant::now` equivalent.  For `quanta`, `raw` mode and `now` are as described above.

We can see that both `quanta` and `clocksource` are measurably faster than `Instant::now` both in taking the discrete measurements and computing the delta.  `quanta`, however, edges out `clocksource`.

## why use this over stdlib or clocksource?

The performance alone is enough to choose this over the stdlib timing facilities if you're doing performance-critical work or need high-accuracy point-in-time measurements, which `Instant` is just not suitable for.

When compared to `clocksource`, though, we have a few extra features that can make the difference:

- `Clock` can be mocked, allowing you to easily control the passage of time in your tests
- `Clock` provides a `start` and `end` method which, in optimized `asm` mode, can replace calls to `raw` and provide more accuracy in the measurement of the code in between

## license

__quanta__ is licensed under the MIT license. ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
