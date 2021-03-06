#[macro_use]
extern crate mackerel_plugin;
#[macro_use]
extern crate serde_json;

use mackerel_plugin::{Graph, Plugin};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::time;

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

#[test]
fn graph_has_diff() {
    let graph1 = graph! {
        name: "sample.foobar",
        label: "Foo bar",
        unit: "integer",
        metrics: [
            { name: "foo", label: "Foo metric" },
            { name: "bar", label: "Bar metric", stacked: true },
            { name: "baz", label: "Baz metric", diff: true },
            { name: "qux", label: "Qux metric", stacked: true, diff: true },
        ]
    };
    assert_eq!(graph1.has_diff(), true);
    let graph2 = graph! {
        name: "sample.foobar",
        label: "Foo bar",
        unit: "integer",
        metrics: [
            { name: "foo", label: "Foo metric" },
            { name: "bar", label: "Bar metric", stacked: true },
            { name: "baz", label: "Baz metric", diff: false },
            { name: "qux", label: "Qux metric", stacked: true, diff: false },
        ]
    };
    assert_eq!(graph2.has_diff(), false);
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
        vec![graph! {
            name: "dice",
            label: "My Dice",
            unit: "integer",
            metrics: [
                { name: "d6", label: "Die 6" },
                { name: "d20", label: "Die 20" }
            ]
        }]
    }
}

fn current_epoch() -> i64 {
    let now = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("error");
    if now.subsec_millis() < 900 {
        now.as_secs() as i64
    } else {
        std::thread::sleep(std::time::Duration::from_millis(100));
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("error")
            .as_secs() as i64
    }
}

#[test]
fn plugin_output_values() {
    let plugin = DicePlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out), Ok(()));
    assert_eq!(
        String::from_utf8(out.into_inner()).unwrap(),
        format!(
            "{}\t{}\t{}\n{}\t{}\t{}\n",
            "dice.d6", 3.0, now, "dice.d20", 17.0, now
        )
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
        serde_json::from_str::<serde_json::Value>(
            out_str.chars().skip(24).collect::<String>().as_ref()
        )
        .unwrap(),
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
    assert_eq!(plugin.output_values(&mut out), Ok(()));
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(
        out_str.contains(&format!(
            "{}\t{}\t{}\n",
            "inode.percentage.sda1.used", 48.2, now
        )),
        true
    );
    assert_eq!(
        out_str.contains(&format!(
            "{}\t{}\t{}\n",
            "inode.percentage.sda-2_1Z.used", 63.7, now
        )),
        true
    );
    assert_eq!(out_str.contains("inode.percentage.sda3.used.etc"), false);
    assert_eq!(out_str.contains("inode.percentage..used"), false);
    assert_eq!(
        out_str.contains(&format!(
            "{}\t{}\t{}\n",
            "inode.count.sda1.used", 1212333.0, now
        )),
        true
    );
    assert_eq!(
        out_str.contains(&format!(
            "{}\t{}\t{}\n",
            "inode.count.sda1.total", 2515214.0, now
        )),
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
        serde_json::from_str::<serde_json::Value>(
            out_str.chars().skip(24).collect::<String>().as_ref()
        )
        .unwrap(),
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
        vec![graph! {
            name: "percentage.#",
            label: "Inode percentage",
            unit: "percentage",
            metrics: [
                { name: "used", label: "used %" },
            ]
        }]
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
    assert_eq!(plugin.output_values(&mut out), Ok(()));
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(
        out_str.contains(&format!(
            "{}\t{}\t{}\n",
            "inode.percentage.sda1.used", 48.2, now
        )),
        true
    );
    assert_eq!(
        out_str.contains(&format!(
            "{}\t{}\t{}\n",
            "inode.percentage.sda2.used", 63.7, now
        )),
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
        serde_json::from_str::<serde_json::Value>(
            out_str.chars().skip(24).collect::<String>().as_ref()
        )
        .unwrap(),
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

struct UptimePlugin {}

impl Plugin for UptimePlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        metrics.insert("uptime".to_string(), 123456.0);
        metrics.insert("foobar".to_string(), 456789.0);
        Ok(metrics)
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![graph! {
            name: "",
            label: "Uptime",
            unit: "integer",
            metrics: [
                { name: "uptime", label: "uptime" },
            ]
        }]
    }

    fn metric_key_prefix(&self) -> String {
        "uptime".to_string()
    }
}

#[test]
fn empty_graph_name_plugin_output_values() {
    let plugin = UptimePlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out), Ok(()));
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(
        out_str.contains(&format!("{}\t{}\t{}\n", "uptime.uptime", 123456.0, now)),
        true
    );
    assert_eq!(out_str.contains("uptime.foobar"), false);
}

#[test]
fn empty_graph_name_plugin_output_definitions() {
    let plugin = UptimePlugin {};
    let mut out = Cursor::new(Vec::new());
    assert_eq!(plugin.output_definitions(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(out_str.starts_with("# mackerel-agent-plugin\n"), true);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            out_str.chars().skip(24).collect::<String>().as_ref()
        )
        .unwrap(),
        json!({
            "graphs": {
                "uptime": {
                    "label": "Uptime",
                    "metrics": [
                        { "name": "uptime", "label": "uptime", "stacked": false },
                    ],
                    "unit": "integer",
                },
            }
        })
    );
}

struct DiffMetricPlugin {}

impl Plugin for DiffMetricPlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        metrics.insert("foobar.diff".to_string(), now.as_secs() as f64);
        metrics.insert("foobar.nodiff".to_string(), 100.0);
        metrics.insert("baz.qux.diff".to_string(), 3.0 * now.as_secs() as f64);
        metrics.insert("baz.qux.nodiff".to_string(), 300.0);
        Ok(metrics)
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![
            graph! {
                name: "foobar",
                label: "Diff graph",
                unit: "integer",
                metrics: [
                    { name: "diff", label: "diff", diff: true },
                    { name: "nodiff", label: "nodiff", diff: false },
                ]
            },
            graph! {
                name: "baz.*",
                label: "Wildcard diff graph",
                unit: "integer",
                metrics: [
                    { name: "diff", label: "diff", diff: true },
                    { name: "nodiff", label: "nodiff", diff: false },
                ]
            },
        ]
    }
}

#[test]
fn diff_metric_plugin_output_values() {
    let plugin = DiffMetricPlugin {};
    let _ = fs::remove_file(plugin.tempfile_path(""));
    let now = current_epoch();
    {
        let mut out = Cursor::new(Vec::new());
        assert_eq!(plugin.output_values(&mut out), Ok(()));
        let out_str = String::from_utf8(out.into_inner()).unwrap();
        assert_eq!(out_str.contains("foobar.diff"), false);
        assert_eq!(
            out_str.contains(&format!("{}\t{}\t{}\n", "foobar.nodiff", 100.0, now)),
            true
        );
        assert_eq!(out_str.contains("baz.qux.diff"), false);
        assert_eq!(
            out_str.contains(&format!("{}\t{}\t{}\n", "baz.qux.nodiff", 300.0, now)),
            true
        );
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    let now = now + 1;
    {
        let mut out = Cursor::new(Vec::new());
        assert_eq!(plugin.output_values(&mut out), Ok(()));
        let out_str = String::from_utf8(out.into_inner()).unwrap();
        assert_eq!(
            out_str.contains(&format!("{}\t{}\t{}\n", "foobar.diff", 60.0, now)),
            true
        );
        assert_eq!(
            out_str.contains(&format!("{}\t{}\t{}\n", "foobar.nodiff", 100.0, now)),
            true
        );
        assert_eq!(
            out_str.contains(&format!("{}\t{}\t{}\n", "baz.qux.diff", 180.0, now)),
            true
        );
        assert_eq!(
            out_str.contains(&format!("{}\t{}\t{}\n", "baz.qux.nodiff", 300.0, now)),
            true
        );
    }
    let _ = fs::remove_file(plugin.tempfile_path(""));
}
