use serde_derive::{Deserialize, Serialize};

use crate::metric::Metric;
use crate::unit::Unit;

/// A graph represents a Mackerel graph schema.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    #[serde(skip_serializing)]
    pub name: String,
    pub label: String,
    pub unit: Unit,
    pub metrics: Vec<Metric>,
}

impl Graph {
    #[doc(hidden)]
    pub fn has_diff(&self) -> bool {
        self.metrics.iter().any(|metric| metric.diff)
    }
}

/// Builds a new [`Graph`].
///
/// ```rust
/// use mackerel_plugin::graph;
///
/// let graph = graph! {
///     name: "linux.swap",
///     label: "Linux Swap Usage",
///     unit: "integer",
///     metrics: [
///         { name: "pswpin", label: "Swap In", diff: true },
///         { name: "pswpout", label: "Swap Out", diff: true },
///     ],
/// };
/// ```
#[macro_export]
macro_rules! graph {
    (
        name: $name:expr,
        label: $label:expr,
        unit: $unit:expr,
        metrics: [$( {$( $metrics:tt )*} ),+ $(,)?] $(,)?
    ) => {{
        assert!(
            str::chars($name).all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '*' | '#'))
                && !$name.starts_with('.') && !$name.ends_with('.')
        );
        $crate::Graph {
            name: $name.into(),
            label: $label.into(),
            unit: $unit.parse().unwrap(),
            metrics: vec![$( $crate::metric! {$( $metrics )*} ),+],
        }
    }};

    ($($_:tt)*) => {
        compile_error!("name, label, unit, and metrics are required");
    };
}
