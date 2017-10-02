#[macro_use]
extern crate mackerel_plugin;
#[macro_use]
extern crate serde_json;
extern crate time;

use std::collections::HashMap;
use std::io::Cursor;
use mackerel_plugin::{Graph, Plugin};

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
    assert_eq!(out_str.starts_with("# mackerel-agent-plugin\n"), true);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(out_str.chars().skip(24).collect::<String>().as_ref()).unwrap(),
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

struct InodePlugin {}

impl Plugin for InodePlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        metrics.insert("inode.count.sda1.used".to_string(), 1212333.0);
        metrics.insert("inode.count.sda1.total".to_string(), 2515214.0);
        metrics.insert("inode.count.sda2.used".to_string(), 1212334.0);
        metrics.insert("inode.count.sda2.total".to_string(), 2515215.0);
        metrics.insert("inode.percentage.sda1.used".to_string(), 48.2);
        metrics.insert("inode.percentage.sda-2_1Z.used".to_string(), 63.7);
        metrics.insert("inode.percentage.sda3.used.etc".to_string(), 72.1);
        metrics.insert("inode.percentage..used".to_string(), 36.2);
        Ok(metrics)
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![
            graph! {
                name: "inode.percentage.#",
                label: "Inode percentage",
                unit: "percentage",
                metrics: [
                    { name: "used", label: "used %" },
                ]
            },
            graph! {
                name: "inode.count.sda1",
                label: "sda1",
                unit: "integer",
                metrics: [
                    { name: "*", label: "%1" },
                ]
            },
        ]
    }
}

#[test]
fn wildcard_plugin_output_values() {
    let plugin = InodePlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "inode.percentage.sda1.used", 48.2, now)),
        true
    );
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "inode.percentage.sda-2_1Z.used", 63.7, now)),
        true
    );
    assert_eq!(out_str.contains("inode.percentage.sda3.used.etc"), false);
    assert_eq!(out_str.contains("inode.percentage..used"), false);
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "inode.count.sda1.used", 1212333.0, now)),
        true
    );
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "inode.count.sda1.total", 2515214.0, now)),
        true
    );
    assert_eq!(out_str.contains("inode.count.sda2.used"), false);
}

#[test]
fn wildcard_plugin_output_definitions() {
    let plugin = InodePlugin {};
    let mut out = Cursor::new(Vec::new());
    assert_eq!(plugin.output_definitions(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(out_str.starts_with("# mackerel-agent-plugin\n"), true);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(out_str.chars().skip(24).collect::<String>().as_ref()).unwrap(),
        json!({
            "graphs": {
                "inode.percentage.#": {
                    "label": "Inode percentage",
                    "metrics": [
                        { "name": "used", "label": "used %", "stacked": false },
                    ],
                    "unit": "percentage"
                },
                "inode.count.sda1": {
                    "label": "sda1",
                    "metrics": [
                        { "name": "*", "label": "%1", "stacked": false },
                    ],
                    "unit": "integer"
                }
            }
        })
    );
}

struct PrefixPlugin {}

impl Plugin for PrefixPlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        metrics.insert("count.sda1.used".to_string(), 1212333.0);
        metrics.insert("count.sda1.total".to_string(), 2515214.0);
        metrics.insert("percentage.sda1.used".to_string(), 48.2);
        metrics.insert("percentage.sda2.used".to_string(), 63.7);
        metrics.insert("percentage.sda3.used.etc".to_string(), 72.1);
        metrics.insert("percentage..used".to_string(), 36.2);
        Ok(metrics)
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![
            graph! {
                name: "percentage.#",
                label: "Inode percentage",
                unit: "percentage",
                metrics: [
                    { name: "used", label: "used %" },
                ]
            },
        ]
    }

    fn metric_key_prefix(&self) -> String {
        "inode".to_string()
    }
}

#[test]
fn prefix_plugin_output_values() {
    let plugin = PrefixPlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "inode.percentage.sda1.used", 48.2, now)),
        true
    );
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "inode.percentage.sda2.used", 63.7, now)),
        true
    );
    assert_eq!(out_str.contains("inode.count.sda1.used"), false);
    assert_eq!(out_str.contains("inode.percentage.sda3.used.etc"), false);
    assert_eq!(out_str.contains("inode.percentage..used"), false);
}

#[test]
fn prefix_plugin_output_definitions() {
    let plugin = PrefixPlugin {};
    let mut out = Cursor::new(Vec::new());
    assert_eq!(plugin.output_definitions(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(out_str.starts_with("# mackerel-agent-plugin\n"), true);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(out_str.chars().skip(24).collect::<String>().as_ref()).unwrap(),
        json!({
            "graphs": {
                "inode.percentage.#": {
                    "label": "Inode percentage",
                    "metrics": [
                        { "name": "used", "label": "used %", "stacked": false },
                    ],
                    "unit": "percentage",
                },
            }
        })
    );
}
