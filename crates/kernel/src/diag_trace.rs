//! Observation-only diagnostic trace for boolean/pattern decisions.
//!
//! Active only when `SOLIDCRAFT_TRACE_BOOLEANS=1`. With the variable unset this
//! module does nothing and the call sites are no-ops (the `enabled()` check is
//! a single cached load). Output goes to the path in `SOLIDCRAFT_TRACE_OUT`
//! (one file per seed/run), one line per decision.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static INIT: std::sync::Once = std::sync::Once::new();

static SEED: AtomicU64 = AtomicU64::new(0);
static FEATURE: AtomicUsize = AtomicUsize::new(0);
// We can't store a &str in an atomic easily; keep a fixed-size buffer.
static OP: Mutex<String> = Mutex::new(String::new());

static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn enabled() -> bool {
    INIT.call_once(|| {
        let on = std::env::var("SOLIDCRAFT_TRACE_BOOLEANS").ok().as_deref() == Some("1");
        ENABLED.store(on, Ordering::Relaxed);
        if on {
            if let Ok(path) = std::env::var("SOLIDCRAFT_TRACE_OUT") {
                if let Ok(f) = OpenOptions::new().create(true).append(true).open(&path) {
                    *FILE.lock().unwrap() = Some(f);
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

pub fn emit(
    predicate: &str,
    computed: f64,
    threshold: f64,
    branch: &str,
    faces: usize,
    edges: usize,
    verts: usize,
) {
    if !enabled() {
        return;
    }
    let seed = SEED.load(Ordering::Relaxed);
    let feat = FEATURE.load(Ordering::Relaxed);
    let op = OP.lock().map(|g| g.clone()).unwrap_or_default();
    let line = format!(
        "seed={} feature={} op={} pred={} computed={:.17e} (0x{:016x}) threshold={:.17e} branch={} faces={} edges={} verts={}\n",
        seed, feat, op, predicate, computed, computed.to_bits(), threshold, branch, faces, edges, verts
    );
    if let Ok(mut g) = FILE.lock() {
        if let Some(f) = g.as_mut() {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }
}