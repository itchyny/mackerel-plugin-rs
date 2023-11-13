use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;

use mackerel_plugin::{graph, Graph, Plugin};

struct DicePlugin {}

impl Plugin for DicePlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        Ok(HashMap::from([
            ("dice.d6".to_owned(), 3.0),
            ("dice.d20".to_owned(), 17.0),
        ]))
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
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("error");
    if now.subsec_millis() < 900 {
        now.as_secs() as i64
    } else {
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
    assert!(plugin.output_definitions(&mut out).is_ok());
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert!(out_str.starts_with("# mackerel-agent-plugin\n"));
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
        Ok(HashMap::from([
            ("inode.count.sda1.used".to_owned(), 1212333.0),
            ("inode.count.sda1.total".to_owned(), 2515214.0),
            ("inode.count.sda2.used".to_owned(), 1212334.0),
            ("inode.count.sda2.total".to_owned(), 2515215.0),
            ("inode.percentage.sda1.used".to_owned(), 48.2),
            ("inode.percentage.sda-2_1Z.used".to_owned(), 63.7),
            ("inode.percentage.sda3.used.etc".to_owned(), 72.1),
            ("inode.percentage..used".to_owned(), 36.2),
        ]))
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
    assert!(out_str.contains(&format!(
        "{}\t{}\t{}\n",
        "inode.percentage.sda1.used", 48.2, now
    )));
    assert!(out_str.contains(&format!(
        "{}\t{}\t{}\n",
        "inode.percentage.sda-2_1Z.used", 63.7, now
    )));
    assert!(!out_str.contains("inode.percentage.sda3.used.etc"));
    assert!(!out_str.contains("inode.percentage..used"));
    assert!(out_str.contains(&format!(
        "{}\t{}\t{}\n",
        "inode.count.sda1.used", 1212333.0, now
    )));
    assert!(out_str.contains(&format!(
        "{}\t{}\t{}\n",
        "inode.count.sda1.total", 2515214.0, now
    )));
    assert!(!out_str.contains("inode.count.sda2.used"));
}

#[test]
fn wildcard_plugin_output_definitions() {
    let plugin = InodePlugin {};
    let mut out = Cursor::new(Vec::new());
    assert!(plugin.output_definitions(&mut out).is_ok());
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert!(out_str.starts_with("# mackerel-agent-plugin\n"));
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
        Ok(HashMap::from([
            ("count.sda1.used".to_owned(), 1212333.0),
            ("count.sda1.total".to_owned(), 2515214.0),
            ("percentage.sda1.used".to_owned(), 48.2),
            ("percentage.sda2.used".to_owned(), 63.7),
            ("percentage.sda3.used.etc".to_owned(), 72.1),
            ("percentage..used".to_owned(), 36.2),
        ]))
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
        "inode".to_owned()
    }
}

#[test]
fn prefix_plugin_output_values() {
    let plugin = PrefixPlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out), Ok(()));
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert!(out_str.contains(&format!(
        "{}\t{}\t{}\n",
        "inode.percentage.sda1.used", 48.2, now
    )));
    assert!(out_str.contains(&format!(
        "{}\t{}\t{}\n",
        "inode.percentage.sda2.used", 63.7, now
    )));
    assert!(!out_str.contains("inode.count.sda1.used"));
    assert!(!out_str.contains("inode.percentage.sda3.used.etc"));
    assert!(!out_str.contains("inode.percentage..used"));
}

#[test]
fn prefix_plugin_output_definitions() {
    let plugin = PrefixPlugin {};
    let mut out = Cursor::new(Vec::new());
    assert!(plugin.output_definitions(&mut out).is_ok());
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert!(out_str.starts_with("# mackerel-agent-plugin\n"));
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
        Ok(HashMap::from([
            ("uptime".to_owned(), 123456.0),
            ("foobar".to_owned(), 456789.0),
        ]))
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
        "uptime".to_owned()
    }
}

#[test]
fn empty_graph_name_plugin_output_values() {
    let plugin = UptimePlugin {};
    let mut out = Cursor::new(Vec::new());
    let now = current_epoch();
    assert_eq!(plugin.output_values(&mut out), Ok(()));
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert!(out_str.contains(&format!("{}\t{}\t{}\n", "uptime.uptime", 123456.0, now)));
    assert!(!out_str.contains("uptime.foobar"));
}

#[test]
fn empty_graph_name_plugin_output_definitions() {
    let plugin = UptimePlugin {};
    let mut out = Cursor::new(Vec::new());
    assert!(plugin.output_definitions(&mut out).is_ok());
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert!(out_str.starts_with("# mackerel-agent-plugin\n"));
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        Ok(HashMap::from([
            ("foobar.diff".to_owned(), now.as_secs() as f64),
            ("foobar.nodiff".to_owned(), 100.0),
            ("baz.qux.diff".to_owned(), 3.0 * now.as_secs() as f64),
            ("baz.qux.nodiff".to_owned(), 300.0),
        ]))
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
    let _ = std::fs::remove_file(plugin.tempfile_path(""));
    let now = current_epoch();
    {
        let mut out = Cursor::new(Vec::new());
        assert_eq!(plugin.output_values(&mut out), Ok(()));
        let out_str = String::from_utf8(out.into_inner()).unwrap();
        assert!(!out_str.contains("foobar.diff"));
        assert!(out_str.contains(&format!("{}\t{}\t{}\n", "foobar.nodiff", 100.0, now)));
        assert!(!out_str.contains("baz.qux.diff"));
        assert!(out_str.contains(&format!("{}\t{}\t{}\n", "baz.qux.nodiff", 300.0, now)));
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    let now = now + 1;
    {
        let mut out = Cursor::new(Vec::new());
        assert_eq!(plugin.output_values(&mut out), Ok(()));
        let out_str = String::from_utf8(out.into_inner()).unwrap();
        assert!(out_str.contains(&format!("{}\t{}\t{}\n", "foobar.diff", 60.0, now)));
        assert!(out_str.contains(&format!("{}\t{}\t{}\n", "foobar.nodiff", 100.0, now)));
        assert!(out_str.contains(&format!("{}\t{}\t{}\n", "baz.qux.diff", 180.0, now)));
        assert!(out_str.contains(&format!("{}\t{}\t{}\n", "baz.qux.nodiff", 300.0, now)));
    }
    let _ = std::fs::remove_file(plugin.tempfile_path(""));
}
