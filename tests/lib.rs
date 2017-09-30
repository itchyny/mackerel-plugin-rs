extern crate serde_json;

extern crate mackerel_plugin;

use std::collections::HashMap;
use std::io::Cursor;
use mackerel_plugin::*;

#[test]
fn serialize_graph() {
    let graph = Graph::new(
        "foo.bar".to_string(),
        "Foo bar".to_string(),
        Unit::Integer,
        vec![
            Metric::new("foo".to_string(), "Foo metric".to_string()),
            Metric::new("bar".to_string(), "Bar metric".to_string()).stacked(),
            Metric::new("baz".to_string(), "Baz metric".to_string()).diff(),
            Metric::new("qux".to_string(), "Qux metric".to_string()).diff().stacked(),
        ],
    );
    let json: serde_json::Value = serde_json::from_str(
        r##"{
              "label": "Foo bar",
              "metrics": [
                { "name": "foo", "label": "Foo metric", "stacked": false },
                { "name": "bar", "label": "Bar metric", "stacked": true },
                { "name": "baz", "label": "Baz metric", "stacked": false },
                { "name": "qux", "label": "Qux metric", "stacked": true }
              ],
              "unit": "integer"
            }"##,
    ).unwrap();
    assert_eq!(serde_json::to_value(graph).unwrap(), json);
}

struct DicePlugin {}

impl Plugin for DicePlugin {
    fn fetch_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("dice.d6".to_string(), 3.0);
        metrics.insert("dice.d20".to_string(), 17.0);
        metrics
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![
            Graph::new(
                "dice".to_string(),
                "My Dice".to_string(),
                Unit::Integer,
                vec![
                    Metric::new("d6".to_string(), "Die 6".to_string()),
                    Metric::new("d20".to_string(), "Die 20".to_string()),
                ],
            ),
        ]
    }
}
