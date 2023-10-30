use serde_derive::{Deserialize, Serialize};

/// A metric represents a Mackerel metric schema.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub label: String,
    pub stacked: bool,
    #[serde(skip_serializing)]
    pub diff: bool,
}

/// Builds a new [`Metric`].
///
/// ```rust
/// use mackerel_plugin::metric;
///
/// let metric = metric! {
///     name: "foo",
///     label: "Foo metric",
/// };
/// ```
///
/// You can also specify `stacked` and `diff` options.
///
/// ```rust
/// use mackerel_plugin::metric;
///
/// let metric = metric! {
///     name: "foo",
///     label: "Foo metric",
///     stacked: true,
///     diff: true,
/// };
/// ```
#[macro_export]
macro_rules! metric {
    (
        name: $name:expr,
        label: $label:expr
        $( , $field:ident: $value:expr )* $(,)?
    ) => {{
        assert!(
            !$name.is_empty()
                && (str::chars($name).all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'))
                    || matches!($name, "*" | "#"))
        );
        $crate::Metric {
            $( $field: $value, )*
            ..$crate::Metric {
                name: $name.into(),
                label: $label.into(),
                stacked: false,
                diff: false,
            }
        }
    }};

    ($($_:tt)*) => {
        compile_error!("name and label are required");
    };
}
