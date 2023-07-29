fog-schemars
============

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/Cognoscan/fog-schemars)
[![Cargo](https://img.shields.io/crates/v/fog-schemars.svg)](https://crates.io/crates/fog-schemars)
[![Documentation](https://docs.rs/fog-schemars/badge.svg)](https://docs.rs/fog-schemars)

It's like [Schemars][Schemars] but for [fog-pack][fog-pack]! This crate lets you 
quickly generate fog-pack validators for Rust types, and then bundle them into 
schemas. It also provides a way to perform one-time initialization of a schema, 
which can then be reliably used anywhere.

[fog-pack]: https://github.com/Cognoscan/fog-pack
[Schemars]: https://github.com/GREsau/schemars
