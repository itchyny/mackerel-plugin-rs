#[macro_use]
extern crate mackerel_plugin;
#[macro_use]
extern crate serde_json;
extern crate time;

use std::collections::HashMap;
use std::io::Cursor;
use mackerel_plugin::*;

#[test]
fn serialize_graph() {
    let graph = graph! {
        name: "foo.bar",
        label: "Foo bar",
        unit: "bytes/sec",
        metrics: [
            { name: "foo", label: "Foo metric" },
            { name: "bar", label: "Bar metric", stacked: true },
            { name: "baz", label: "Baz metric", diff: true },
            { name: "qux", label: "Qux metric", stacked: true, diff: true },
        ]
    };
    let json = json!({
        "label": "Foo bar",
        "metrics": [
            { "name": "foo", "label": "Foo metric", "stacked": false },
            { "name": "bar", "label": "Bar metric", "stacked": true },
            { "name": "baz", "label": "Baz metric", "stacked": false },
            { "name": "qux", "label": "Qux metric", "stacked": true }
        ],
        "unit": "bytes/sec"
    });
    assert_eq!(serde_json::to_value(graph).unwrap(), json);
}

struct DicePlugin {}

impl Plugin for DicePlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        metrics.insert("dice.d6".to_string(), 3.0);
        metrics.insert("dice.d20".to_string(), 17.0);
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
                    { name: "d20", label: "Die 20" }
                ]
            },
        ]
    }
}

fn current_epoch() -> i64 {
    if time::now().tm_nsec > 900_000_000 {
        std::thread::sleep(std::time::Duration::from_millis(100))
    }
    time::now().to_timespec().sec
}

#[test]
fn plugin_output_values() {
    let plugin = DicePlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out).is_ok(), true);
    assert_eq!(
        String::from_utf8(out.into_inner()).unwrap(),
        format!("{}\t{}\t{}\n{}\t{}\t{}\n", "dice.d6", 3.0, now, "dice.d20", 17.0, now)
    );
}

#[test]
fn plugin_output_definitions() {
    let plugin = DicePlugin {};
    let mut out = Cursor::new(Vec::new());
    assert_eq!(plugin.output_definitions(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(out_str.starts_with("# mackerel-agent-plugins\n"), true);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(out_str.chars().skip(25).collect::<String>().as_ref()).unwrap(),
        json!({
            "graphs": {
                "dice": {
                    "label": "My Dice",
                    "metrics": [
                        { "name": "d6", "label": "Die 6", "stacked": false },
                        { "name": "d20", "label": "Die 20", "stacked": false }
                    ],
                    "unit": "integer"
                }
            }
        })
    );
}
