//! Observation-only diagnostic trace for boolean/pattern decisions.
//!
//! Active only when `SOLIDCRAFT_TRACE_BOOLEANS=1`. With the variable unset this
//! module does nothing and the call sites are no-ops (the `enabled()` check is
//! a single cached load of an atomic). Output goes to the path in
//! `SOLIDCRAFT_TRACE_OUT` (one file per run), one line per decision.
//!
//! Every value is written as `0x<bits>/decimal` so a platform diff is exact.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static INIT: std::sync::Once = std::sync::Once::new();

static SEED: AtomicU64 = AtomicU64::new(0);
static FEATURE: AtomicUsize = AtomicUsize::new(0);
static OP: Mutex<String> = Mutex::new(String::new());

static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn enabled() -> bool {
    INIT.call_once(|| {
        let on = std::env::var("SOLIDCRAFT_TRACE_BOOLEANS").ok().as_deref() == Some("1");
        ENABLED.store(on, Ordering::Relaxed);
        if on && let Ok(path) = std::env::var("SOLIDCRAFT_TRACE_OUT") {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(&path) {
                if let Ok(mut g) = FILE.lock() {
                    *g = Some(f);
                }
            }
        }
    });
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_context(seed: u64, feature_index: usize, op_name: &str) {
    if !enabled() {
        return;
    }
    SEED.store(seed, Ordering::Relaxed);
    FEATURE.store(feature_index, Ordering::Relaxed);
    if let Ok(mut g) = OP.lock() {
        g.clear();
        g.push_str(op_name);
    }
}

fn context() -> (u64, usize, String) {
    (SEED.load(Ordering::Relaxed), FEATURE.load(Ordering::Relaxed), OP.lock().map(|g| g.clone()).unwrap_or_default())
}

fn write_line(line: &str) {
    if let Ok(mut g) = FILE.lock() {
        if let Some(f) = g.as_mut() {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }
}

/// One value as exact bits and as decimal.
fn val(v: f64) -> String {
    format!("0x{:016x}/{:.17e}", v.to_bits(), v)
}

/// One decision: computed value, threshold, branch, and the body counts at that point.
pub fn emit(predicate: &str, computed: f64, threshold: f64, branch: &str, faces: usize, edges: usize, verts: usize) {
    if !enabled() {
        return;
    }
    let (seed, feat, op) = context();
    let line = format!(
        "seed={seed} feature={feat} op={op} pred={predicate} computed={} threshold={} branch={branch} faces={faces} edges={edges} verts={verts}\n",
        val(computed),
        val(threshold)
    );
    write_line(&line);
}

/// One decision with a named set of values, for decisions that carry more than
/// a single computed value and threshold.
pub fn emit_fields(predicate: &str, branch: &str, fields: &[(&str, f64)], counts: (usize, usize, usize)) {
    if !enabled() {
        return;
    }
    let (seed, feat, op) = context();
    let mut fs = String::new();
    for (name, v) in fields {
        if !fs.is_empty() {
            fs.push(',');
        }
        fs.push_str(name);
        fs.push('=');
        fs.push_str(&val(*v));
    }
    let (faces, edges, verts) = counts;
    let line = format!("seed={seed} feature={feat} op={op} pred={predicate} branch={branch} fields={fs} faces={faces} edges={edges} verts={verts}\n");
    write_line(&line);
}
