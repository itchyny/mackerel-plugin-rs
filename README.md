# Mackerel plugin helper library for Rust

```rust
#[macro_use]
extern crate mackerel_plugin;
extern crate rand;

use std::collections::HashMap;
use mackerel_plugin::*;

struct DicePlugin {}

impl Plugin for DicePlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        metrics.insert("dice.d6".to_string(), (rand::random::<u64>() % 6 + 1) as f64);
        metrics.insert("dice.d20".to_string(), (rand::random::<u64>() % 20 + 1) as f64);
        Ok(metrics)
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
                ]
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
