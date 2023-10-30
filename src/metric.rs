use serde_derive::{Deserialize, Serialize};

/// A Metric
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub label: String,
    pub stacked: bool,
    #[serde(skip_serializing)]
    pub diff: bool,
}

impl Metric {
    pub fn new(name: String, label: String, stacked: bool, diff: bool) -> Metric {
        if name.is_empty()
            || !(name
                .chars()
                .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'))
                || name == "*"
                || name == "#")
        {
            panic!("invalid metric name: {}", name);
        }
        Metric {
            name,
            label,
            stacked,
            diff,
        }
    }
}

/// Construct a Metric.
///
/// ```rust
/// use mackerel_plugin::metric;
///
/// let metric = metric! {
///     name: "foo",
///     label: "Foo metric"
/// };
/// ```
///
/// Additionally you can specify `stacked` and `diff` options.
///
/// ```rust
/// use mackerel_plugin::metric;
///
/// let metric = metric! {
///     name: "foo",
///     label: "Foo metric",
///     stacked: true,
///     diff: true
/// };
/// ```
#[macro_export]
macro_rules! metric {
    (name: $name:expr, label: $label:expr) => {
        $crate::Metric::new($name.into(), $label.into(), false, false)
    };

    (name: $name:expr, label: $label:expr, stacked: $stacked:expr) => {
        $crate::Metric::new($name.into(), $label.into(), $stacked, false)
    };

    (name: $name:expr, label: $label:expr, diff: $diff:expr) => {
        $crate::Metric::new($name.into(), $label.into(), false, $diff)
    };

    (name: $name:expr, label: $label:expr, stacked: $stacked:expr, diff: $diff:expr) => {
        $crate::Metric::new($name.into(), $label.into(), $stacked, $diff)
    };

    ($($token:tt)*) => {
        compile_error!("name and label are required for a metric");
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! metrics {
    ($({$($token:tt)+},)*) => {
        vec![$($crate::metric! {$($token)+},)*]
    };

    ($({$($token:tt)+}),*) => {
        vec![$($crate::metric! {$($token)+}),*]
    };
}
