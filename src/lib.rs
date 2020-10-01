use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::time;

extern crate rand;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

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
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
            _ => false,
        }
    };
}

impl Metric {
    pub fn new(name: String, label: String, stacked: bool, diff: bool) -> Metric {
        if name.is_empty()
            || !(name.chars().all(|c| valid_chars!(c)) || name == "*" || name == "#")
            || name.starts_with(".")
            || name.ends_with(".")
        {
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
        if !name
            .chars()
            .all(|c| valid_chars!(c) || c == '.' || c == '*' || c == '#')
            || name.starts_with(".")
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

    #[doc(hidden)]
    pub fn has_diff(&self) -> bool {
        self.metrics.iter().any(|metric| metric.diff)
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

#[derive(Debug, Serialize, Deserialize)]
struct MetricValues {
    timestamp: i64,
    values: HashMap<String, f64>,
}

impl MetricValues {
    fn new(timestamp: i64, values: HashMap<String, f64>) -> MetricValues {
        MetricValues {
            timestamp: timestamp,
            values: values,
        }
    }

    fn empty() -> MetricValues {
        MetricValues {
            timestamp: 0,
            values: HashMap::new(),
        }
    }
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
    fn output_values(&self, out: &mut dyn io::Write) -> Result<(), String> {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let metric_values = MetricValues::new(now.as_secs() as i64, self.fetch_metrics()?);
        let prefix = self.metric_key_prefix();
        let graphs = self.graph_definition();
        let has_diff = graphs.iter().any(|graph| graph.has_diff());
        let path = self.tempfile_path(&prefix);
        let prev_metric_values = if has_diff {
            load_values(&path).unwrap_or(MetricValues::empty())
        } else {
            MetricValues::empty()
        };
        for graph in graphs {
            for metric in graph.metrics {
                format_values(
                    out,
                    &prefix,
                    &graph.name,
                    metric,
                    &metric_values,
                    &prev_metric_values,
                );
            }
        }
        if has_diff {
            save_values(&path, &metric_values)?;
        }
        Ok(())
    }

    #[doc(hidden)]
    fn tempfile_path(&self, prefix: &str) -> String {
        let name = if prefix.is_empty() {
            let arg0 = env::args().next().unwrap();
            let exec_name = path::Path::new(&arg0)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            if exec_name.starts_with("mackerel-plugin-") {
                exec_name.to_string()
            } else {
                "mackerel-plugin-".to_string() + exec_name
            }
        } else {
            "mackerel-plugin-".to_string() + &prefix
        };
        env::var("MACKEREL_PLUGIN_WORKDIR")
            .map_or(env::temp_dir(), |path| path::PathBuf::from(&path))
            .join(name)
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[doc(hidden)]
    fn output_definitions(&self, out: &mut dyn io::Write) -> Result<(), String> {
        writeln!(out, "# mackerel-agent-plugin").map_err(|e| format!("{}", e))?;
        let prefix = self.metric_key_prefix();
        let json = json!({
            "graphs": self.graph_definition()
                .iter()
                .map(|graph|
                    (
                        if prefix.is_empty() {
                            graph.name.clone()
                        } else if graph.name.is_empty() {
                            prefix.clone()
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

    fn run(&self) -> Result<(), String> {
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        if env::var("MACKEREL_AGENT_PLUGIN_META").map_or(false, |value| value != "") {
            self.output_definitions(&mut out)
        } else {
            self.output_values(&mut out)
        }
    }
}

fn load_values(path: &str) -> Result<MetricValues, String> {
    let file = fs::File::open(path).map_err(|e| format!("open {} failed: {}", path, e))?;
    serde_json::de::from_reader(file).map_err(|e| format!("read {} failed: {}", path, e))
}

fn save_values(path: &str, metric_values: &MetricValues) -> Result<(), String> {
    let bytes = serde_json::to_vec(metric_values).unwrap();
    atomic_write(path, bytes.as_slice(), metric_values.timestamp)
}

fn atomic_write(path: &str, bytes: &[u8], now: i64) -> Result<(), String> {
    let tmp_path = &format!("{}.{}{}", path, now, rand::random::<u64>());
    let mut file =
        fs::File::create(tmp_path).map_err(|e| format!("open {} failed: {}", tmp_path, e))?;
    file.write(bytes)
        .map_err(|e| format!("write to {} failed: {}", tmp_path, e))?;
    drop(file);
    fs::rename(tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(tmp_path);
        format!("rename {} to {} failed: {}", tmp_path, path, e)
    })
}

fn format_values(
    out: &mut dyn io::Write,
    prefix: &str,
    graph_name: &str,
    metric: Metric,
    metric_values: &MetricValues,
    prev_metric_values: &MetricValues,
) {
    for (metric_name, value) in
        collect_metric_values(graph_name, metric, metric_values, prev_metric_values)
    {
        if !value.is_nan() && value.is_finite() {
            let name = if prefix.is_empty() {
                metric_name
            } else {
                prefix.to_string() + "." + metric_name.as_ref()
            };
            writeln!(out, "{}\t{}\t{}", name, value, metric_values.timestamp).unwrap();
        }
    }
}

fn collect_metric_values(
    graph_name: &str,
    metric: Metric,
    metric_values: &MetricValues,
    prev_metric_values: &MetricValues,
) -> Vec<(String, f64)> {
    let is_diff = metric.diff;
    let metric_name = if graph_name.is_empty() {
        metric.name
    } else {
        format!("{}.{}", graph_name, &metric.name)
    };
    let count = metric_name.chars().filter(|c| *c == '.').count();
    if metric_name.contains("*") || metric_name.contains("#") {
        metric_values
            .values
            .iter()
            .filter(|&(name, _)| {
                name.chars().filter(|c| *c == '.').count() == count
                    && metric_name.split('.').zip(name.split('.')).all(|(cs, ds)| {
                        if cs == "*" || cs == "#" {
                            !ds.is_empty() && ds.chars().all(|c| valid_chars!(c))
                        } else {
                            cs == ds
                        }
                    })
            })
            .filter_map(|(metric_name, value)| {
                (if is_diff {
                    prev_metric_values
                        .values
                        .get(metric_name)
                        .and_then(|prev_value| {
                            calc_diff(
                                *value,
                                metric_values.timestamp,
                                *prev_value,
                                prev_metric_values.timestamp,
                            )
                        })
                } else {
                    Some(*value)
                })
                .map(|value| (metric_name.clone(), value))
            })
            .collect()
    } else {
        metric_values
            .values
            .get(&metric_name)
            .and_then(|value| {
                if is_diff {
                    prev_metric_values
                        .values
                        .get(&metric_name)
                        .and_then(|prev_value| {
                            calc_diff(
                                *value,
                                metric_values.timestamp,
                                *prev_value,
                                prev_metric_values.timestamp,
                            )
                        })
                } else {
                    Some(*value)
                }
            })
            .map(|value| (metric_name, value))
            .into_iter()
            .collect()
    }
}

#[inline]
fn calc_diff(value: f64, timestamp: i64, prev_value: f64, prev_timestamp: i64) -> Option<f64> {
    if prev_timestamp < timestamp - 600 || timestamp <= prev_timestamp || prev_value > value {
        None
    } else {
        Some((value - prev_value) / ((timestamp - prev_timestamp) as f64 / 60.0))
    }
}
