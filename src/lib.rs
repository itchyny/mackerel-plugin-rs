use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io;

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

/// A Metric
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    name: String,
    label: String,
    stacked: bool,
    #[serde(skip_serializing)]
    diff: bool,
}

impl Metric {
    pub fn new(name: String, label: String) -> Metric {
        Metric {
            name: name,
            label: label,
            stacked: false,
            diff: false,
        }
    }

    pub fn stacked(&self) -> Metric {
        Metric {
            stacked: true,
            ..self.clone()
        }
    }

    pub fn diff(&self) -> Metric {
        Metric {
            diff: true,
            ..self.clone()
        }
    }
}

#[macro_export]
macro_rules! metric {
    (name: $name:expr, label: $label:expr) => {
        Metric::new($name.into(), $label.into())
    };

    (name: $name:expr, label: $label:expr, stacked: $stacked:expr) => {
        Metric::new($name.into(), $label.into()).stacked()
    };

    (name: $name:expr, label: $label:expr, diff: $diff:expr) => {
        Metric::new($name.into(), $label.into()).diff()
    };

    (name: $name:expr, label: $label:expr, stacked: $stacked:expr, diff: $diff:expr) => {
        Metric::new($name.into(), $label.into()).stacked().diff()
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
        Graph {
            name: name,
            label: label,
            unit: unit,
            metrics: metrics,
        }
    }
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
