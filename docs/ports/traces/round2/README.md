# Round-2 traces: robustness goldens seeds 13 and 67

Decision traces for `tests::robustness_goldens::seed_13` and `seed_67`, one file per seed per
platform, all collected at commit `104d6913b8891d7a6732ea127c0f0ed2d5bd1629` (`diag/golden-trace`).
The hook is `crates/kernel/src/diag_trace.rs`; it is inert unless `SOLIDCRAFT_TRACE_BOOLEANS=1`
(report §3 has the observation-only evidence). Analysis is in
[`docs/ports/golden-trace-report.md`](../golden-trace-report.md).

Line format, one line per predicate evaluation:

```
seed=<n> feature=<n> op=<name> pred=<name> branch=<name> fields=<name>=0x<hex>/<decimal>,... faces=<n> edges=<n> verts=<n>
```

The hook appends, so delete the target file before a run.

## Committed files

| platform | file | committed bytes | committed SHA-256 |
|---|---|---|---|
| x86_64 Linux | `x86_64-linux/trace_x86_64_seed13.txt` | 931,884 | `6f78e7d97a85b04468fca40935cc0e65032a5598c818d6a9eda821100d3c6e23` |
| x86_64 Linux | `x86_64-linux/trace_x86_64_seed67.txt` | 143,050 (first 400 lines) | `c61c9fcdc28e73dcefaa9276f626ba8902677064a545fc215fea69f3ecad71b6` |
| aarch64 Linux | `aarch64-linux/trace_aarch64_seed13.txt` | 932,209 | `c2b2558c1a621f38dfaa5a4a3759d023296e8b8c465b094d27dbe434cb464146` |
| aarch64 Linux | `aarch64-linux/trace_aarch64_seed67.txt` | 143,050 (first 400 lines) | `c61c9fcdc28e73dcefaa9276f626ba8902677064a545fc215fea69f3ecad71b6` |
| Windows ARM64 | `aarch64-windows/trace_aarch64_win_seed13.txt` | 945,333 | `93aa8a7c3c213ab2a4a6f60d15a1c6e70cb76ca3955687c476ac79a5f07f53b9` |
| Windows ARM64 | `aarch64-windows/trace_aarch64_win_seed67.txt` | 143,072 (first 400 lines) | `2bfc603caf81a2b598ce468b6f4ece95c17b6c1a584255d22fc41cde570d1a4e` |

The seed 67 files exceed the 1 MB commit limit, so the first 400 lines are committed (seed 67's
first divergence is line 125, well inside the trim). Full files:

| platform | full bytes | full-file SHA-256 |
|---|---|---|
| x86_64 Linux | 1,227,206 | `0b892f35ee51de0fc4ba83c517fba58ab801ce37c21340e7798ffeb2a993e499` |
| aarch64 Linux | 1,227,206 | `0b892f35ee51de0fc4ba83c517fba58ab801ce37c21340e7798ffeb2a993e499` (byte-identical to x86_64) |
| Windows ARM64 | 1,227,463 | `f20e0d64187e3be59b35d8cc3753941b732e54d51541df2aeafd7bb700695dc6` |

The x86_64 and aarch64 Linux seed 67 runs are byte-identical (the same hash for the committed
400-line files follows from that).

## How each file was produced

- **x86_64 Linux**: GitHub Actions `ubuntu-latest`, run
  <https://github.com/varunkumar898/solidcraft/actions/runs/39145480504>, workflow
  `.github/workflows/golden-trace.yml`; artifacts `trace-x86-64-seed13` / `trace-x86-64-seed67`,
  each also carrying `COMMIT_SHA.txt` (checked to be the commit above). 7-day retention.
- **aarch64 Linux**: native Ubuntu 22.04 under WSL2 on the Windows ARM64 host, the tree synced
  with `git archive 104d6913 | tar -x -C /home/varun/solidcraft` (the mirror has no `.git`).
- **Windows ARM64**: native `aarch64-pc-windows-gnullvm` build on the same host.

Command per platform (delete the output file first; the golden assertion itself fails in these
runs because the stored references are stale — report §2 — the traces are still complete):

```powershell
$env:SOLIDCRAFT_TRACE_BOOLEANS = "1"
$env:SOLIDCRAFT_TRACE_OUT = "C:\Users\...\trace_aarch64_win_seed13.txt"
cargo test -p solidcraft-engine --lib "tests::robustness_goldens::seed_13" -- --nocapture
```

```bash
SOLIDCRAFT_TRACE_BOOLEANS=1 SOLIDCRAFT_TRACE_OUT=/home/varun/trace_aarch64_seed13.txt \
  cargo test -p solidcraft-engine --lib "tests::robustness_goldens::seed_13" -- --nocapture
```

(and `seed_67` for each). Independent reruns are bit-identical; setting the variable does not
change any branch or value (report §3).
