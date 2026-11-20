# Changelog
## [v0.3.0](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.6..v0.3.0) (2026-11-20)
* use the hash of the command line arguments in the file name of metric values, so that
  plugins with different arguments do not share the values
* fix error handling on writing metric values, and return errors instead of panicking
* fix atomic write of the metric values file to write all the bytes
* fix to construct the file path of metric values only when there are diff metrics
* evaluate the name argument only once in `graph!` and `metric!` macros
* update Rust edition to 2024 (requires Rust 1.88)

## [v0.2.6](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.5..v0.2.6) (2023-11-14)
* add `seconds`, `milliseconds`, and `bits/sec` units
* fix atomic write of the metric values file for diff metrics, and remove dependency on
  `rand` crate
* improve error handling on constructing the temporary file path
* fix repetition operator in `graph!` macro
* update Rust edition to 2021, and update dependencies

## [v0.2.5](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.4..v0.2.5) (2020-10-03)
* use `env::var` instead of `env::vars` to get environment variable values
* update dependencies

## [v0.2.4](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.3..v0.2.4) (2020-07-18)
* remove dependency on `time` crate
* update dependencies

## [v0.2.3](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.2..v0.2.3) (2018-11-10)
* update dependencies

## [v0.2.2](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.1..v0.2.2) (2017-10-08)
* implement diff metrics
* use `io::BufWriter` in `Plugin::run`

## [v0.2.1](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.2.0..v0.2.1) (2017-10-04)
* implement support for `metric_key_prefix`, and empty graph name with `metric_key_prefix`

## [v0.2.0](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.1.0..v0.2.0) (2017-10-01)
* implement support for wildcard metrics
* add more validation for graph name and metric name

## [v0.1.0](https://github.com/itchyny/mackerel-plugin-rs/compare/v0.0.0..v0.1.0) (2017-10-01)
* fix the first line in the output of graph definitions

## [v0.0.0](https://github.com/itchyny/mackerel-plugin-rs/compare/a3f6a7f..v0.0.0) (2017-10-01)
* initial implementation of the helper library for Mackerel agent plugins
* implement `Plugin` trait, `graph!` and `metric!` macros, and metric units
