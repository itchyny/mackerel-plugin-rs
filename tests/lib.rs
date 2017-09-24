extern crate serde_json;

extern crate mackerel_plugin;

use mackerel_plugin::*;

#[test]
fn serialize_graph() {
    let graph = Graph::new(
        "foo.bar".into(),
        "Foo bar".into(),
        Unit::Integer,
        vec![
            Metric::new("foo".into(), "Foo metric".into()),
            Metric::new("bar".into(), "Bar metric".into()).stacked(),
            Metric::new("baz".into(), "Baz metric".into()).diff(),
            Metric::new("qux".into(), "Qux metric".into()).diff().stacked(),
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
