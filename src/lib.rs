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

// Compile time validation?
macro_rules! name_validation {
    ($type:expr, $name:expr) => {
        if !$name.chars().all(|c| {
            'a' <= c && c <= 'z' || 'A' <= c && c <= 'Z' || '0' <= c && c <= '9' || c == '-' || c == '_' || c == '.'
        }) {
            panic!("invalid {} name: {}", $type, $name);
        }
    }
}

impl Metric {
    pub fn new(name: String, label: String, stacked: bool, diff: bool) -> Metric {
        name_validation!("metric", name);
        Metric {
            name: name,
            label: label,
            stacked: stacked,
            diff: diff,
        }
    }
}

#[macro_export]
macro_rules! metric {
    (name: $name:expr, label: $label:expr) => {
        Metric::new($name.into(), $label.into(), false, false)
    };

    (name: $name:expr, label: $label:expr, stacked: $stacked:expr) => {
        Metric::new($name.into(), $label.into(), $stacked, false)
    };

    (name: $name:expr, label: $label:expr, diff: $diff:expr) => {
        Metric::new($name.into(), $label.into(), false, $diff)
    };

    (name: $name:expr, label: $label:expr, stacked: $stacked:expr, diff: $diff:expr) => {
        Metric::new($name.into(), $label.into(), $stacked, $diff)
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
        name_validation!("graph", name);
        Graph {
            name: name,
            label: label,
            unit: unit,
            metrics: metrics,
        }
    }
}

#[macro_export]
macro_rules! graph {
    (name: $name:expr, label: $label:expr, unit: $unit:expr, metrics: [$($metrics:tt)+]) => {
        Graph::new($name.into(), $label.into(), $unit.into(), metrics!($($metrics)+))
    };

    ($($token:tt)*) => {
        compile_error!("name, label, unit and metrics are required for a graph");
    };
}

/// A Plugin
pub trait Plugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String>;

    fn graph_definition(&self) -> Vec<Graph>;

    fn print_value(&self, out: &mut io::Write, metric_name: String, value: f64, now: time::Timespec) {
        if !value.is_nan() && value.is_finite() {
            let _ = writeln!(out, "{}\t{}\t{}", metric_name, value, now.sec);
        }
    }

    fn output_values(&self, out: &mut io::Write) -> Result<(), String> {
        let now = time::now().to_timespec();
        let results = self.fetch_metrics()?;
        for graph in self.graph_definition() {
            for metric in graph.metrics {
                self.format_values(out, &graph.name, metric, &results, now);
            }
        }
        Ok(())
    }

    fn format_values(&self, out: &mut io::Write, graph_name: &str, metric: Metric, results: &HashMap<String, f64>, now: time::Timespec) {
        let metric_name = format!("{}.{}", graph_name, &metric.name);
        results.get(&metric_name).map(|value| self.print_value(out, metric_name, *value, now));
    }

    fn output_definitions(&self, out: &mut io::Write) -> Result<(), String> {
        writeln!(out, "# mackerel-agent-plugins").map_err(|e| format!("{}", e))?;
        let json = json!({"graphs": self.graph_definition().iter().map(|graph| (&graph.name, graph)).collect::<HashMap<_, _>>()});
        writeln!(out, "{}", json.to_string()).map_err(|e| format!("{}", e))?;
        Ok(())
    }

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
