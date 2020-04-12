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
- monotonic time in nanoseconds or raw cycles
- extremely low overhead where possible
- optimized for instruction-level accuracy in measurements
- mockable!
- cross-platform!
- fun, science-y name!

## platform / architecture support

For platforms, we have tier 1 support for Linux, Windows, and macOS/iOS.  Platforms such as Solaris or various BSDs has tier 2.5 support: `quanta` should work on them by virtue of depending on `libc`, but we don't test or build on these platforms as all.

Architecture-wise, x86/x86-64 and SSE2 are required for the optimized TSC codepath.  This is handled transparently via compile-time target features, so you must build with the appropriate compiler flags to specify the CPU features where your binary will run, as runtime detection is not supported.

## performance

Accessing the TSC on a modern x86 processor has an extremely low overhead of roughly ~11ns, and `quanta` provides the thinnest possible layer over this.  Using the native time facilities, such as `clock_gettime(CLOCK_MONOTONIC)` on Linux, you may expect to see closer to 17-18ns of overhead.

Measurements have not been taken for non-x86-based architectures/platforms.

## why use this over stdlib or clocksource?

The performance alone is enough to choose this over the stdlib timing facilities if you're doing performance-critical work or need high-accuracy point-in-time measurements, which `Instant` is just not suitable for.

When compared to `clocksource`, though, we have a few extra features that can make the difference:

- `Clock` can be mocked, allowing you to easily control the passage of time in your tests
- `Clock` provides `start` and `end` as replacements for `raw`, which are optimized for instruction-level accuracy, avoiding instruction reordering that might taint measurements

## license

__quanta__ is licensed under the MIT license. ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
