use auto_enums::auto_enum;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;

use crate::graph::Graph;
use crate::metric::Metric;

#[derive(Default, Serialize, Deserialize)]
struct MetricValues {
    timestamp: i64,
    values: HashMap<String, f64>,
}

impl MetricValues {
    fn new(timestamp: i64, values: HashMap<String, f64>) -> MetricValues {
        MetricValues { timestamp, values }
    }
}

/// A trait which represents a Plugin.
///
/// You can create a plugin by implementing `fetch_metrics` and `graph_definition`.
pub trait Plugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String>;

    fn graph_definition(&self) -> Vec<Graph>;

    fn metric_key_prefix(&self) -> String {
        "".to_owned()
    }

    #[doc(hidden)]
    fn output_values(&self, out: &mut dyn std::io::Write) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let metric_values = MetricValues::new(now.as_secs() as i64, self.fetch_metrics()?);
        let prefix = self.metric_key_prefix();
        let graphs = self.graph_definition();
        let has_diff = graphs.iter().any(|graph| graph.has_diff());
        let path = self.tempfile_path(&prefix);
        let prev_metric_values = if has_diff {
            load_values(&path).unwrap_or_default()
        } else {
            MetricValues::default()
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
            let arg0 = std::env::args().next().unwrap();
            let exec_name = std::path::Path::new(&arg0)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            if exec_name.starts_with("mackerel-plugin-") {
                exec_name.to_owned()
            } else {
                "mackerel-plugin-".to_owned() + exec_name
            }
        } else {
            "mackerel-plugin-".to_owned() + prefix
        };
        std::env::var("MACKEREL_PLUGIN_WORKDIR")
            .map_or(std::env::temp_dir(), |path| std::path::PathBuf::from(&path))
            .join(name)
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[doc(hidden)]
    fn output_definitions(&self, out: &mut dyn std::io::Write) -> Result<(), String> {
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
        writeln!(out, "{}", json).map_err(|e| format!("{}", e))?;
        Ok(())
    }

    fn run(&self) -> Result<(), String> {
        let stdout = std::io::stdout();
        let mut out = std::io::BufWriter::new(stdout.lock());
        if std::env::var("MACKEREL_AGENT_PLUGIN_META").map_or(false, |value| !value.is_empty()) {
            self.output_definitions(&mut out)
        } else {
            self.output_values(&mut out)
        }
    }
}

fn load_values(path: &str) -> Result<MetricValues, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open {} failed: {}", path, e))?;
    serde_json::de::from_reader(file).map_err(|e| format!("read {} failed: {}", path, e))
}

fn save_values(path: &str, metric_values: &MetricValues) -> Result<(), String> {
    let bytes = serde_json::to_vec(metric_values).unwrap();
    atomic_write(path, bytes.as_slice())
}

fn atomic_write(path: &str, bytes: &[u8]) -> Result<(), String> {
    let tmp_path = &format!(
        "{}.{}",
        path,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs_f64()
    );
    let mut file =
        std::fs::File::create(tmp_path).map_err(|e| format!("open {} failed: {}", tmp_path, e))?;
    file.write(bytes)
        .map_err(|e| format!("write to {} failed: {}", tmp_path, e))?;
    drop(file);
    std::fs::rename(tmp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(tmp_path);
        format!("rename {} to {} failed: {}", tmp_path, path, e)
    })
}

fn format_values(
    out: &mut dyn std::io::Write,
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
                prefix.to_owned() + "." + metric_name.as_ref()
            };
            writeln!(out, "{}\t{}\t{}", name, value, metric_values.timestamp).unwrap();
        }
    }
}

#[auto_enum(Iterator)]
fn collect_metric_values<'a>(
    graph_name: &'a str,
    metric: Metric,
    metric_values: &'a MetricValues,
    prev_metric_values: &'a MetricValues,
) -> impl Iterator<Item = (String, f64)> + 'a {
    let metric_name = if graph_name.is_empty() {
        metric.name
    } else {
        graph_name.to_owned() + "." + &metric.name
    };
    let count = metric_name.chars().filter(|&c| c == '.').count();
    if metric_name.contains('*') || metric_name.contains('#') {
        metric_values
            .values
            .iter()
            .filter(move |&(name, _)| {
                name.chars().filter(|&c| c == '.').count() == count
                    && metric_name.split('.').zip(name.split('.')).all(|(cs, ds)| {
                        if cs == "*" || cs == "#" {
                            !ds.is_empty()
                                && ds.chars().all(
                                    |c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'),
                                )
                        } else {
                            cs == ds
                        }
                    })
            })
            .filter_map(move |(metric_name, &value)| {
                if metric.diff {
                    prev_metric_values
                        .values
                        .get(metric_name)
                        .and_then(|&prev_value| {
                            calc_diff(
                                value,
                                metric_values.timestamp,
                                prev_value,
                                prev_metric_values.timestamp,
                            )
                        })
                } else {
                    Some(value)
                }
                .map(|value| (metric_name.clone(), value))
            })
    } else {
        metric_values
            .values
            .get(&metric_name)
            .and_then(|&value| {
                if metric.diff {
                    prev_metric_values
                        .values
                        .get(&metric_name)
                        .and_then(|&prev_value| {
                            calc_diff(
                                value,
                                metric_values.timestamp,
                                prev_value,
                                prev_metric_values.timestamp,
                            )
                        })
                } else {
                    Some(value)
                }
            })
            .map(|value| (metric_name, value))
            .into_iter()
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
