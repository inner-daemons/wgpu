use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

pub const SCHEMA: u32 = 1;

/// One measurement of one case, for example the SPIR-V writer on one entry point.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Record {
    /// `cpu_compile`, `pipeline_creation` or `gpu_runtime`.
    pub bench: String,
    /// `bench`, `example` or `snapshot`.
    pub group: String,
    /// The shader, entry point or pipeline measured.
    pub case: String,
    /// What was measured, for example `spv`, `create_pipeline` or `gpu_time`.
    pub metric: String,
    pub unit: String,
    /// `ok`, `error`, `panic`, `crash` or `skipped`.
    pub status: String,
    pub error: Option<String>,
    /// Number of samples.
    pub n: u32,
    pub median: f64,
    pub mean: f64,
    pub p90: f64,
    pub min: f64,
    pub max: f64,
    pub aux: BTreeMap<String, String>,
}

impl Record {
    pub fn ok(
        bench: &str,
        group: &str,
        case: &str,
        metric: &str,
        unit: &str,
        samples: &mut [f64],
    ) -> Self {
        samples.sort_by(f64::total_cmp);
        let n = samples.len();
        let (median, mean, p90, min, max) = if n == 0 {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        } else {
            let at = |q: f64| samples[((q * n as f64).ceil() as usize).clamp(1, n) - 1];
            (
                if n % 2 == 1 {
                    samples[n / 2]
                } else {
                    (samples[n / 2 - 1] + samples[n / 2]) / 2.0
                },
                samples.iter().sum::<f64>() / n as f64,
                at(0.9),
                samples[0],
                samples[n - 1],
            )
        };
        Self {
            bench: bench.to_string(),
            group: group.to_string(),
            case: case.to_string(),
            metric: metric.to_string(),
            unit: unit.to_string(),
            status: "ok".to_string(),
            error: None,
            n: n as u32,
            median,
            mean,
            p90,
            min,
            max,
            aux: BTreeMap::new(),
        }
    }

    pub fn failed(
        bench: &str,
        group: &str,
        case: &str,
        metric: &str,
        status: &str,
        error: impl Into<String>,
    ) -> Self {
        Self {
            bench: bench.to_string(),
            group: group.to_string(),
            case: case.to_string(),
            metric: metric.to_string(),
            status: status.to_string(),
            error: Some(error.into()),
            ..Self::default()
        }
    }

    pub fn aux(mut self, key: &str, value: impl ToString) -> Self {
        self.aux.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ResultFile {
    pub schema: u32,
    pub bench: String,
    /// The case being measured when the file was last written. If it is still set when the
    /// benchmark starts again with `NAGA_PASSES_RESUME=1`, the process died during that case.
    pub in_progress: Option<String>,
    pub meta: BTreeMap<String, String>,
    pub records: Vec<Record>,
}

pub fn out_dir() -> PathBuf {
    match std::env::var_os("NAGA_PASSES_OUT") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../target/bench/naga_passes"
        )),
    }
}

/// Writes the result file after every case, so partial results survive a crash.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    file: ResultFile,
    done: BTreeSet<String>,
}

impl Store {
    pub fn open(bench: &str) -> Self {
        let dir = out_dir();
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(format!("{bench}.json"));

        let mut file = ResultFile {
            schema: SCHEMA,
            bench: bench.to_string(),
            ..ResultFile::default()
        };
        if std::env::var_os("NAGA_PASSES_RESUME").is_some() {
            if let Some(old) = fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<ResultFile>(&bytes).ok())
            {
                file = old;
            }
        }

        if let Some(case) = file.in_progress.take() {
            file.records.push(Record::failed(
                bench,
                "unknown",
                &case,
                "total",
                "crash",
                "the process exited while measuring this case",
            ));
        }

        let done = file.records.iter().map(|r| r.case.clone()).collect();
        let mut store = Self { path, file, done };
        store.write();
        store
    }

    pub fn is_done(&self, case: &str) -> bool {
        self.done.contains(case)
    }

    pub fn set_meta(&mut self, key: &str, value: impl ToString) {
        self.file.meta.insert(key.to_string(), value.to_string());
    }

    pub fn begin(&mut self, case: &str) {
        self.file.in_progress = Some(case.to_string());
        self.write();
    }

    pub fn finish(&mut self, case: &str, records: Vec<Record>) {
        self.file.in_progress = None;
        self.file.records.extend(records);
        self.done.insert(case.to_string());
        self.write();
    }

    pub fn records(&self) -> &[Record] {
        &self.file.records
    }

    pub fn write(&mut self) {
        let tmp = self.path.with_extension("json.tmp");
        let result = serde_json::to_vec(&self.file)
            .map_err(|e| e.to_string())
            .and_then(|bytes| fs::write(&tmp, bytes).map_err(|e| e.to_string()))
            .and_then(|()| fs::rename(&tmp, &self.path).map_err(|e| e.to_string()));
        if let Err(e) = result {
            eprintln!("  Could not write {}: {e}", self.path.display());
        }
    }
}
