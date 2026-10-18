# Golden trace report: robustness_goldens seeds 13 and 67 (Linux aarch64 vs Windows ARM64 proxy)

## Setup

- Branch: `diag/golden-trace` (from `port/arm64`).
- Hook: `crates/kernel/src/diag_trace.rs`, off unless `SOLIDCRAFT_TRACE_BOOLEANS=1`.
- Context set in `fuzz::run_session` (seed, feature index, op name).
- Instrumented: `boolean()` result (input vol, output vol, branch, face/edge counts), `polybool::split` near-boundary vertices (`|t| < 2*eps`), and `faceted_boolean` tolerance sweep.

## Step 4 verification (hook is observation-only)

Ran `debug_seed_13_steps` on Windows ARM64 (gnullvm) with the env var unset, then set.
The step-by-step volume/area/face/edge/shell numbers were identical in both runs
(e.g. final: vol=2396.939546 area=2101.521015 faces=40 edges=101). The hook is
observation-only: yes, with this evidence.

## aarch64 traces (WSL Ubuntu 22.04, native)

- `docs/ports/traces/aarch64/trace_aarch64_seed13.txt` (12 lines)
- `docs/ports/traces/aarch64/trace_aarch64_seed67.txt` (428 lines)

## x86_64 traces

Could not be collected: GitHub only registers workflows that exist on the default
branch (`main`), and the task forbids pushing to `main`. The workflow file
`.github/workflows/golden-trace.yml` is committed on `diag/golden-trace` and
runs on `ubuntu-latest`. To collect x86_64 traces, merge the workflow to `main`
(or run it from a PR) and trigger `workflow_dispatch` with seed 13 and 67.
Artifact names: `trace-x86-64-seed13`, `trace-x86-64-seed67` (7-day retention).

## Diff (Windows ARM64 gnullvm proxy vs Linux aarch64)

Seed 13, first differing line is line 1 of the trace:
- Windows: `computed=2.06163244355585812e3 (0x40a01b43cfa449f4)`
- aarch64: `computed=2.06163244355585857e3 (0x40a01b43cfa449f5)`
- Difference: 1 ulp (last bit). Branch taken: `some` on both. faces/edges identical (10/4).
- All 12 boolean_result lines show the same branch on both platforms; the
  computed values differ only in the last bit (1 ulp) throughout.
- `split_near_boundary` and `faceted_boolean_tol_sweep` produced zero lines on
  both platforms (the booleans in these seeds go through `boolean_whole` / truck,
  not the faceted/planar BSP path).

## Interpretation

The first divergence is a 1-ulp floating-point difference in the boolean input
volume (`va`), present from the very first boolean. The branch taken is identical
on both platforms, so this is a value difference, not a control-flow flip, at the
traced predicate level. However, the final topology differs (edges 103->99,
faces 41->39 on aarch64), which a 1-ulp volume difference alone cannot explain.
The tolerance-sensitive decision that flips the topology is in code that was NOT
traced: `boolean_whole` (truck's B-rep boolean) and/or `planar_boolean`'s
internal predicates. Those need instrumentation to locate the exact flip.

## What was NOT traced

- `boolean_whole` (truck B-rep boolean): no trace hooks (truck is an external
  crate, cannot be instrumented without a wrapper).
- `planar_boolean` internal predicates (coplanar tests, face classification):
  not instrumented.
- `split` in `polybool.rs`: instrumented but produced zero near-boundary hits,
  meaning either the booleans didn't use the BSP path or no vertex sat within
  2*eps of a splitting plane.
- `faceted_boolean` tolerance sweep: instrumented but produced zero hits (seeds
  use exact planar booleans, not faceting).

## Commands run

- aarch64 trace collection:
  `SOLIDCRAFT_TRACE_BOOLEANS=1 SOLIDCRAFT_TRACE_OUT=trace_aarch64_seed13.txt cargo test -p solidcraft-engine --lib debug_seed_13_steps -- --nocapture`
- Windows (gnullvm) trace collection:
  `SOLIDCRAFT_TRACE_BOOLEANS=1 SOLIDCRAFT_TRACE_OUT=C:/Users/varun/trace_win.txt cargo test -p solidcraft-engine --lib debug_seed_13_steps -- --nocapture`
- Diff: compared line-by-line; first difference at line 1.

## Files

- `docs/ports/traces/aarch64/trace_aarch64_seed13.txt` (12 lines, 2042 bytes)
- `docs/ports/traces/aarch64/trace_aarch64_seed67.txt` (428 lines, 79266 bytes)
- `docs/ports/golden-trace-plan.md`
- `.github/workflows/golden-trace.yml`