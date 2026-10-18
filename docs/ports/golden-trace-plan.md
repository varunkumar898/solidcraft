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

## Round 2, Step 1: do the golden and the debug print measure the same quantity?

Yes — same quantity, verified on one machine (Windows ARM64, this branch).

- The golden asserts `fuzz::golden(seed, 8)` (`crates/engine/src/tests/robustness_goldens.rs`):
  volume/area within relative 1e-6, and `faces`/`edges`/`shells` compared **exactly** against
  `tests/robustness/goldens.json` (with `platforms.windows` overrides applied when
  `std::env::consts::OS == "windows"`).
- `fuzz::golden` (`crates/engine/src/fuzz.rs`) sums `solidcraft_kernel::measure(&b.body)` over
  `s.model.state().bodies`: `m.volume`, `m.area`, `m.faces`, `m.edges`, `m.shells`.
- The debug test `tests::debug_seed_13_steps` (`crates/engine/src/tests.rs`) replays
  `fuzz::run(13, 8)` into a fresh `Session` and sums **the same fields of the same
  `measure()`** over the final bodies.
- Evidence they agree: on Windows ARM64 the actual golden test fails with
  `seed 13: edges: 103 -> 101; faces: 41 -> 40` and the debug replay prints
  `faces=40 edges=101` — the same numbers from the two code paths.
- So the cross-platform count data are comparable:

| source | seed 13 faces/edges | seed 67 faces/edges |
|---|---|---|
| `goldens.json` shared (recorded from CI, x86_64 Linux) | 41 / 103 | 37 / 88 |
| `goldens.json` `platforms.windows` override | — (none) | 36 / 86 |
| Windows ARM64 (this branch, golden test + debug replay) | 40 / 101 (FAIL) | 36 / 86 (PASS, override) |
| Linux aarch64 (recorded failure message) | 39 / 99 (FAIL) | 36 / 86 (FAIL vs shared) |

The 40/41 gap is therefore a real per-platform result difference in `measure()`'s exact
topology counts, not a measurement-method difference. (Note `measure().faces/edges` are the raw
kernel counts `b.face_count()`/`b.edge_count()`; `merged` counts exist separately and neither
side uses them.)

## Round 2, Step 2: instrumented points (all observation-only)

Format addition: `emit_fields` lines carry named values as `name=0x<bits>/decimal` so a
platform diff is exact. Per-element emissions from hash-map iteration are collected and sorted
first (the maps are met in random hash order; #707 keeps the result independent of it and the
trace must be too).

| point | file | predicate | what is recorded |
|---|---|---|---|
| step parameters | engine/src/fuzz.rs `trace_params` | `step_param` | every f64 of every command's params, as bits, before the step |
| per-step totals | engine/src/fuzz.rs `trace_step` | `body_summary`, `step_summary` | the exact `measure()` sums the golden compares, after every step |
| measurement | kernel/src/measure.rs | `measure_begin`, `measure_mesh`, `measure_tol`, `measure_face` (per face: fine/medium/coarse area, 6·signed volume, the two `close` deltas, pick), `measure_total` | the fields the golden asserts, and their tessellation/selection basis |
| healing, face/edge merge | kernel/src/heal.rs | `heal_begin`, `heal_stage` (before/after faces+edges per stage), `planar_face` (worst deviation vs tol), `fit_exact` (line/circle residuals vs tol), `merge_pair` (dot, |da−db|, keep), `collinear_pair` (dot) | the exact tolerance comparisons that decide face and edge counts |
| boolean route | kernel/src/ops.rs | `bool_route` (conical cut junction, tangent cylinders), `whole_entry` (size, tol_m, va, vb), `whole_route` (prism early/late, trivial sure/doubted), `whole_meshes` (triangle counts, meet, refined tol_m), `check` (bounds/unfused/mismatch fraction) | the route and its inputs |
| truck call boundary | kernel/src/ops.rs `truck_boolean` | `truck_call`, `truck_out` | operand counts, tolerance, shift, result face/edge counts |
| result build | kernel/src/boolean_record.rs | `edges_evaluate`, `heal_build_done` | validity gate and final counts |

Boolean result lines from round 1 (`boolean_result`) are unchanged.