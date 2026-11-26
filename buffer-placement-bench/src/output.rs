//! The human-readable tables and the CSV file.

use std::fmt::Write as _;

use crate::common::Row;

fn format_value(metric: &str, value: f64, unit: &str) -> String {
    match unit {
        "bool" if metric == "check" => if value == 1.0 { "ok" } else { "FAIL" }.into(),
        "bool" => if value == 1.0 { "yes" } else { "no" }.into(),
        "index" if value < 0.0 => "none".into(),
        "index" | "enum" => format!("{value}"),
        "bits" => format!("{:#x}", value as u64),
        "x" => format!("{value:.3}x"),
        _ => format!("{value:.3} {unit}"),
    }
}

/// One line per case and metric, with the new and legacy values side by side
/// and their paired ratio.
pub fn print_table<'a>(rows: impl IntoIterator<Item = &'a Row>, out: &mut String) {
    let rows: Vec<&Row> = rows.into_iter().collect();
    if rows.is_empty() {
        let _ = writeln!(out, "No results.");
        let _ = writeln!(out);
        return;
    }
    // (case, metric) -> [new, legacy, ratio]
    let mut lines: Vec<(&str, &str, [String; 3])> = Vec::new();
    for row in rows {
        let (metric, column) = match row.metric.strip_suffix("_vs_legacy") {
            Some(base) => (base, 2),
            None if row.variant == "legacy" => (row.metric.as_str(), 1),
            None => (row.metric.as_str(), 0),
        };
        let value = format_value(metric, row.value, row.unit);
        match lines
            .iter_mut()
            .find(|(c, m, _)| *c == row.case && *m == metric)
        {
            Some(line) => line.2[column] = value,
            None => {
                let mut cells = [String::new(), String::new(), String::new()];
                cells[column] = value;
                lines.push((&row.case, metric, cells));
            }
        }
    }
    let _ = writeln!(out, "| case | metric | new | legacy | new/legacy |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for (case, metric, [new, legacy, ratio]) in lines {
        let _ = writeln!(out, "| {case} | {metric} | {new} | {legacy} | {ratio} |");
    }
    let _ = writeln!(out);
}

pub fn write_csv(path: &str, rows: &[Row]) {
    let mut csv = String::from("adapter,backend,bench,case,variant,metric,value,unit\n");
    for r in rows {
        let _ = writeln!(
            csv,
            "\"{}\",{},{},\"{}\",{},{},{},{}",
            r.adapter, r.backend, r.bench, r.case, r.variant, r.metric, r.value, r.unit
        );
    }
    std::fs::write(path, csv).expect("failed to write the CSV file");
}
