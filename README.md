# mackerel-plugin-rs
[![CI Status](https://github.com/itchyny/mackerel-plugin-rs/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/itchyny/mackerel-plugin-rs/actions?query=branch:main)
[![crates.io](https://img.shields.io/crates/v/mackerel_plugin.svg)](https://crates.io/crates/mackerel_plugin)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/itchyny/mackerel-plugin-rs/blob/main/LICENSE)

ー Mackerel plugin helper library for Rust ー

## Plugin samples
Plugins using this library.

- [mackerel-plugin-loadavg](https://github.com/itchyny/mackerel-plugin-loadavg)
- [mackerel-plugin-uptime](https://github.com/itchyny/mackerel-plugin-uptime)
- [mackerel-plugin-ntp](https://github.com/itchyny/mackerel-plugin-ntp)
- [mackerel-plugin-dice](https://github.com/itchyny/mackerel-plugin-dice-rs)

## Example
```rust
use mackerel_plugin::*;
use rand;
use std::collections::HashMap;

struct DicePlugin {}

impl Plugin for DicePlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        Ok(HashMap::from([
            ("dice.d6".to_owned(), (rand::random::<u64>() % 6 + 1) as f64),
            ("dice.d20".to_owned(), (rand::random::<u64>() % 20 + 1) as f64),
        ]))
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![
            graph! {
                name: "dice",
                label: "My Dice",
                unit: "integer",
                metrics: [
                    { name: "d6", label: "Die 6" },
                    { name: "d20", label: "Die 20" },
                ],
            },
        ]
    }
}

fn main() {
    let plugin = DicePlugin {};
    match plugin.run() {
        Ok(_) => {},
        Err(err) => {
            eprintln!("mackerel-plugin-dice: {}", err);
            std::process::exit(1);
        }
    }
}
```


## Author
itchyny (https://github.com/itchyny)

## License
This software is released under the MIT License, see LICENSE.
