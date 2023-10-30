use serde_json::json;

use mackerel_plugin::graph;

#[test]
fn graph() {
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
            { name: "qux", label: "Qux metric", diff: true, stacked: true }
        ]
    };
    assert!(graph1.has_diff());

    let graph2 = graph! {
        name: "SAMPLE.FOOBAR",
        label: "Foo bar",
        unit: "percentage",
        metrics: [
            { name: "foo", label: "Foo metric" },
            { name: "bar", label: "Bar metric", stacked: true },
            { name: "baz", label: "Baz metric", diff: false },
            { name: "qux", label: "Qux metric", diff: false, stacked: true },
        ],
    };
    assert!(!graph2.has_diff());
}
