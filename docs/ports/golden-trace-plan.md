# Golden trace plan (diag/golden-trace)

## Test names (verified via `--list`)

Crate: `solidcraft-engine` (lib). Exact filter names:
- `tests::robustness_goldens::seed_13`
- `tests::robustness_goldens::seed_67`

Command: `cargo test -p solidcraft-engine --lib <name> -- --nocapture`

## Decision points to trace (crates/kernel/src/polybool.rs)

| # | file:line | predicate | threshold | notes |
|---|---|---|---|---|
| 1 | polybool.rs:59 | `t < -eps` (split, front/back) | eps = caller-supplied | plane-polygon split |
| 2 | polybool.rs:61 | `t > eps` (split) | eps | |
| 3 | polybool.rs:94 | `den.abs() > 1e-300` | 1e-300 | degenerate plane check |
| 4 | polybool.rs:370-390 | faceted boolean tolerance sweep | [1e-8,1e-7,3e-8,1e-9,3e-7] * size | tries multiple eps, picks by volume match |
| 5 | polybool.rs:390 | `(vo - expect).abs() > 1e-6*(va+vb)` | 1e-6 | volume match check |
| 6 | polybool.rs:390 | `vi < -1e-9*(va+vb)` | 1e-9 | intersection volume check |
| 7 | polybool.rs:252 | `kind_tol`: `(b.size()*1e-7).max(1e-9)*100` | 1e-7/1e-9 | surface classification tolerance |
| 8 | polybool.rs:287 | `tol = (b.size()*1e-3).max(1e-3)` | 1e-3 | planar polys tolerance |

These are the decisions on the boolean/pattern path. The hook instruments (4) and (5) and (6) first (the tolerance sweep), which is the most likely divergence point, then (1)/(2) if needed.

## Trace format

One line per decision when SOLIDCRAFT_TRACE_BOOLEANS=1:
`seed | feature_index | op_name | predicate_name | computed_value (hex to_bits + decimal) | threshold | branch_taken | faces | edges | verts`

Output file: SOLIDCRAFT_TRACE_OUT (one file per seed).

## Branch

diag/golden-trace (from port/arm64). Hook lives in crates/kernel/src/diag_trace.rs.
Off unless SOLIDCRAFT_TRACE_BOOLEANS=1. Zero overhead / identical path when unset.