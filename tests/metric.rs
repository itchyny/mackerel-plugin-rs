use mackerel_plugin::{metric, Metric};

#[test]
fn metric_macro() {
    fn metric(name: &str, label: &str, stacked: bool, diff: bool) -> Metric {
        Metric {
            name: name.to_owned(),
            label: label.to_owned(),
            stacked,
            diff,
        }
    }

    assert_eq!(
        metric! { name: "foo", label: "Foo metric" },
        metric("foo", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", },
        metric("foo", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "abcxyzABCXYZ012789-_", label: "Foo metric" },
        metric("abcxyzABCXYZ012789-_", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "*", label: "Foo metric" },
        metric("*", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "#", label: "Foo metric", },
        metric("#", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", stacked: true },
        metric("foo", "Foo metric", true, false)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", stacked: false },
        metric("foo", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", diff: true },
        metric("foo", "Foo metric", false, true)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", diff: false },
        metric("foo", "Foo metric", false, false)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", stacked: false, diff: true },
        metric("foo", "Foo metric", false, true)
    );
    assert_eq!(
        metric! { name: "foo", label: "Foo metric", diff: false, stacked: true, },
        metric("foo", "Foo metric", true, false)
    );
}
