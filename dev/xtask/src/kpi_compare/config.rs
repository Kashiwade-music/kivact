//! Role: Read the KPI rows of dev/bench/kpi.toml.

/// Whether a row measures time (noise allowance 1.05) or size (1.02).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Time,
    Size,
}

impl Kind {
    /// The largest ratio that still counts as no regression.
    pub(super) fn allowance(self) -> f64 {
        match self {
            Kind::Time => 1.05,
            Kind::Size => 1.02,
        }
    }
}

/// One KPI row of the table.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Row {
    pub(super) name: String,
    pub(super) line: String,
    pub(super) field: String,
    pub(super) kind: Kind,
    pub(super) target: f64,
    pub(super) improve: Option<f64>,
    pub(super) floor: f64,
    pub(super) catalog: bool,
    pub(super) warning: bool,
}

pub(super) fn parse(text: &str) -> Result<Vec<Row>, String> {
    let table: toml::Table = text.parse().map_err(|error| format!("kpi.toml: {error}"))?;
    let rows = table
        .get("row")
        .and_then(toml::Value::as_array)
        .ok_or("kpi.toml: expected [[row]] tables")?;
    rows.iter()
        .enumerate()
        .map(|(index, value)| row(index, value))
        .collect()
}

fn row(index: usize, value: &toml::Value) -> Result<Row, String> {
    let table = value
        .as_table()
        .ok_or_else(|| format!("kpi.toml: row {index} is not a table"))?;
    let text = |key: &str| {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("kpi.toml: row {index}: `{key}` is required"))
    };
    let number = |key: &str| -> Result<Option<f64>, String> {
        table
            .get(key)
            .map(|value| {
                value
                    .as_float()
                    .or_else(|| value.as_integer().map(|integer| integer as f64))
                    .ok_or_else(|| format!("kpi.toml: row {index}: `{key}` must be a number"))
            })
            .transpose()
    };
    let flag = |key: &str| {
        table
            .get(key)
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
    };
    let kind = match text("kind")?.as_str() {
        "time" => Kind::Time,
        "size" => Kind::Size,
        other => return Err(format!("kpi.toml: row {index}: unknown kind {other}")),
    };
    Ok(Row {
        name: text("name")?,
        line: text("line")?,
        field: text("field")?,
        kind,
        target: number("target")?
            .ok_or_else(|| format!("kpi.toml: row {index}: `target` is required"))?,
        improve: number("improve")?,
        floor: number("floor")?.unwrap_or(0.0),
        catalog: flag("catalog"),
        warning: flag("warning"),
    })
}
