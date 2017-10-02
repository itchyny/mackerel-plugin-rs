use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io;
use std::convert::From;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate time;

/// Metric units
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "percentage")]
    Percentage,
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "bytes/sec")]
    BytesPerSecond,
    #[serde(rename = "iops")]
    IOPS,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Unit::Float => write!(f, "float"),
            Unit::Integer => write!(f, "integer"),
            Unit::Percentage => write!(f, "percentage"),
            Unit::Bytes => write!(f, "bytes"),
            Unit::BytesPerSecond => write!(f, "bytes/sec"),
            Unit::IOPS => write!(f, "iops"),
        }
    }
}

impl<'a> From<&'a str> for Unit {
    fn from(src: &str) -> Unit {
        match src {
            "float" => Unit::Float,
            "integer" => Unit::Integer,
            "percentage" => Unit::Percentage,
            "bytes" => Unit::Bytes,
            "bytes/sec" => Unit::BytesPerSecond,
            "iops" => Unit::IOPS,
            x => panic!(
                "invalid unit: {} (should be one of float, integer, percentage, bytes, bytes/sec or iops)",
                x
            ),
        }
    }
}

/// A Metric
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    name: String,
    label: String,
    stacked: bool,
    #[serde(skip_serializing)]
    diff: bool,
}

macro_rules! valid_chars {
    ($c:expr) => {
        match $c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => true,
            _ => false,
        }
    }
}

impl Metric {
    pub fn new(name: String, label: String, stacked: bool, diff: bool) -> Metric {
        if name.len() == 0 || !(name.chars().all(|c| valid_chars!(c)) || name == "*" || name == "#") || name.starts_with(".") || name.ends_with(".") {
            panic!("invalid metric name: {}", name);
        }
        Metric {
            name: name,
            label: label,
            stacked: stacked,
            diff: diff,
        }
    }
}

/// Construct a Metric.
///
/// ```rust
/// # #[macro_use]
/// # extern crate mackerel_plugin;
/// #
/// # fn main() {
/// let metric = metric! {
///     name: "foo",
///     label: "Foo metric"
/// };
/// # }
/// ```
///
/// Additionally you can specify `stacked` and `diff` options.
///
/// ```rust
/// # #[macro_use]
/// # extern crate mackerel_plugin;
/// #
/// # fn main() {
/// let metric = metric! {
///     name: "foo",
///     label: "Foo metric",
///     stacked: true,
///     diff: true
/// };
/// # }
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
        vec![$(metric! {$($token)+},)*]
    };

    ($({$($token:tt)+}),*) => {
        vec![$(metric! {$($token)+}),*]
    };
}

/// A Graph
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    #[serde(skip_serializing)]
    name: String,
    label: String,
    unit: Unit,
    metrics: Vec<Metric>,
}

impl Graph {
    pub fn new(name: String, label: String, unit: Unit, metrics: Vec<Metric>) -> Graph {
        if name.len() == 0 || !name.chars().all(|c| valid_chars!(c) || c == '.' || c == '*' || c == '#') || name.starts_with(".")
            || name.ends_with(".")
        {
            panic!("invalid graph name: {}", name);
        }
        Graph {
            name: name,
            label: label,
            unit: unit,
            metrics: metrics,
        }
    }
}

/// Construct a Graph.
///
/// ```rust
/// # #[macro_use]
/// # extern crate mackerel_plugin;
/// #
/// # fn main() {
/// let graph = graph! {
///     name: "linux.swap",
///     label: "Linux Swap Usage",
///     unit: "integer",
///     metrics: [
///         { name: "pswpin", label: "Swap In", diff: true },
///         { name: "pswpout", label: "Swap Out", diff: true },
///     ]
/// };
/// # }
/// ```
#[macro_export]
macro_rules! graph {
    (name: $name:expr, label: $label:expr, unit: $unit:expr, metrics: [$($metrics:tt)+]) => {
        $crate::Graph::new($name.into(), $label.into(), $unit.into(), metrics!($($metrics)+))
    };

    ($($token:tt)*) => {
        compile_error!("name, label, unit and metrics are required for a graph");
    };
}

/// A trait which represents a Plugin.
///
/// You can create a plugin by implementing `fetch_metrics` and `graph_definition`.
pub trait Plugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String>;

    fn graph_definition(&self) -> Vec<Graph>;

    fn metric_key_prefix(&self) -> String {
        "".to_string()
    }

    #[doc(hidden)]
    fn print_value(&self, out: &mut io::Write, metric_name: String, value: f64, now: i64) {
        if !value.is_nan() && value.is_finite() {
            let _ = writeln!(out, "{}\t{}\t{}", metric_name, value, now);
        }
    }

    #[doc(hidden)]
    fn output_values(&self, out: &mut io::Write) -> Result<(), String> {
        let now = time::now().to_timespec().sec;
        let results = self.fetch_metrics()?;
        let prefix = self.metric_key_prefix();
        for graph in self.graph_definition() {
            for metric in graph.metrics {
                self.format_values(out, &prefix, &graph.name, metric, &results, now);
            }
        }
        Ok(())
    }

    #[doc(hidden)]
    fn collect_metric_values(&self, graph_name: &str, metric: Metric, results: &HashMap<String, f64>) -> Vec<(String, f64)> {
        let metric_name = format!("{}.{}", graph_name, &metric.name);
        let count = metric_name.chars().filter(|c| *c == '.').count();
        if metric_name.contains("*") || metric_name.contains("#") {
            results
                .iter()
                .filter(|&(name, _)| {
                    name.chars().filter(|c| *c == '.').count() == count
                        && metric_name.split('.').zip(name.split('.')).all(|(cs, ds)| if cs == "*" || cs == "#" {
                            ds.len() > 0 && ds.chars().all(|c| valid_chars!(c))
                        } else {
                            cs == ds
                        })
                })
                .map(|(metric_name, value)| (metric_name.clone(), *value))
                .collect()
        } else {
            results.get(&metric_name).map(|value| (metric_name, *value)).into_iter().collect()
        }
    }

    #[doc(hidden)]
    fn format_values(&self, out: &mut io::Write, prefix: &str, graph_name: &str, metric: Metric, results: &HashMap<String, f64>, now: i64) {
        for (metric_name, value) in self.collect_metric_values(graph_name, metric, results) {
            let name = if prefix.is_empty() { metric_name } else { prefix.to_string() + "." + metric_name.as_ref() };
            self.print_value(out, name, value, now);
        }
    }

    #[doc(hidden)]
    fn output_definitions(&self, out: &mut io::Write) -> Result<(), String> {
        writeln!(out, "# mackerel-agent-plugin").map_err(|e| format!("{}", e))?;
        let prefix = self.metric_key_prefix();
        let json = json!({
            "graphs": self.graph_definition()
                .iter()
                .map(|graph|
                    (
                        if prefix.is_empty() {
                            graph.name.clone()
                        } else {
                            prefix.clone() + "." + graph.name.as_ref()
                        },
                        graph
                    )
                )
                .collect::<HashMap<_, _>>(),
        });
        writeln!(out, "{}", json.to_string()).map_err(|e| format!("{}", e))?;
        Ok(())
    }

    #[doc(hidden)]
    fn env_plugin_meta(&self) -> Option<String> {
        env::vars()
            .filter_map(|(key, value)| if key == "MACKEREL_AGENT_PLUGIN_META" {
                Some(value)
            } else {
                None
            })
            .next()
    }

    fn run(&self) -> Result<(), String> {
        let mut stdout = io::stdout();
        if self.env_plugin_meta().map_or(false, |value| value != "") {
            self.output_definitions(&mut stdout)
        } else {
            self.output_values(&mut stdout)
        }
    }
}
