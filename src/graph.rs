use serde_derive::{Deserialize, Serialize};

use crate::metric::Metric;
use crate::unit::Unit;

/// A Graph
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    #[serde(skip_serializing)]
    pub name: String,
    pub label: String,
    pub unit: Unit,
    pub metrics: Vec<Metric>,
}

impl Graph {
    pub fn new(name: String, label: String, unit: Unit, metrics: Vec<Metric>) -> Graph {
        if !name
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '*' | '#'))
            || name.starts_with('.')
            || name.ends_with('.')
        {
            panic!("invalid graph name: {}", name);
        }
        Graph {
            name,
            label,
            unit,
            metrics,
        }
    }

    #[doc(hidden)]
    pub fn has_diff(&self) -> bool {
        self.metrics.iter().any(|metric| metric.diff)
    }
}

/// Construct a Graph.
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
///     ]
/// };
/// ```
#[macro_export]
macro_rules! graph {
    (name: $name:expr, label: $label:expr, unit: $unit:expr, metrics: [$($metrics:tt)+]) => {
        $crate::Graph::new($name.into(), $label.into(), $unit.parse().unwrap(), $crate::metrics!($($metrics)+))
    };

    ($($token:tt)*) => {
        compile_error!("name, label, unit and metrics are required for a graph");
    };
}
