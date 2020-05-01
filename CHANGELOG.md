# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2] - 2020-05-01
### Changed
- Fix the logic to figure out when calibration is required. ([#14](https://github.com/metrics-rs/quanta/pull/14))

## [0.5.1] - 2020-04-11
### Changed
- Small tweak to the docs.

## [0.5.0] - 2020-04-11
### Changed
- Switch to `mach` for macOS/iOS as it was deprecated in `libc`. ([#12](https://github.com/metrics-rs/quanta/pull/12))
- Switch to `core::arch` for instrinics, and drop the feature flagged configuration to use it. ([#12](https://github.com/metrics-rs/quanta/pull/12))
- Switch to `criterion` for benchmarking. ([#12](https://github.com/metrics-rs/quanta/pull/12))

## [0.4.0] - 2020-02-20
### Changed
- Differentiate between raw and scaled time by adding a new `Instant` type. ([#10](https://github.com/metrics-rs/quanta/pull/10))

## [0.2.0] - 2019-03-10
### Changed
- Fixed support for Windows.  It was in a bad way, but actually works correctly now!
- Switched to Azure Pipelines CI + Cirrus CI, including formatting, tests, and benchmarks, for Linux, macOS, Windows, and FreeBSD.

## [0.1.0] - 2019-01-14
### Added
- Initial commit.
