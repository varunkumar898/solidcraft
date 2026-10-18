# Golden trace report 2: robustness_goldens seeds 13 and 67 across x86_64 Linux, aarch64 Linux and Windows ARM64

Round 1 of this report built the decision-trace hook and compared aarch64 Linux with
Windows ARM64. This round extends the traced points upstream (base solids, measurement,
healing, the boolean/truck boundary), collects **all three platforms from one commit**, and
answers two questions: the first divergence between x86_64 Linux and aarch64 Linux, and the
first divergence between Windows ARM64 and aarch64 Linux.

- Commit (all three trace sets): `104d6913b8891d7a6732ea127c0f0ed2d5bd1629` on `diag/golden-trace`.
- x86_64 Linux: CI run <https://github.com/varunkumar898/solidcraft/actions/runs/39145480504>
  (`ubuntu-latest`); artifacts `trace-x86-64-seed13` / `trace-x86-64-seed67` (each also holds
  `COMMIT_SHA.txt`; the run's conclusion is `failure` because the seeds' golden assertions
  fail — the traces are still written and uploaded with `if: always()`).
- aarch64 Linux: native Ubuntu 22.04 under WSL2 on the Windows ARM64 host.
- Windows ARM64: native `aarch64-pc-windows-gnullvm` build on the same host.
- Traces committed under `docs/ports/traces/round2/` (sizes and SHA-256 in its README).
- The hook is inert (`diag_trace::emit` returns immediately) unless
  `SOLIDCRAFT_TRACE_BOOLEANS=1`; §3 is the observation-only evidence.
  Line format: `seed | feature | op | predicate | computed (hex + {:e}) | ... | branch | faces | edges | verts`.

## 1. What the three platforms build at this commit

The stored reference is `tests/robustness/goldens.json` as this branch carries it; §2 shows that
file is upstream `main`@`b685b730`'s and why it is stale.

seed 13, faces/edges after each step:

| step | x86_64 Linux | aarch64 Linux | Windows ARM64 |
|---|---|---|---|
| 0 revolved section | 10 / 24 | 10 / 24 | 10 / 24 |
| 1 cut cylinder | 10 / 24 | 10 / 24 | 10 / 24 |
| 2 join box | 20 / 45 | 20 / 45 | 20 / 45 |
| 3 rectangular pattern | 28 / 64 | 28 / 64 | 28 / 64 |
| 4 cut cylinder | 35 / 84 | 35 / 84 | **36 / 86** |
| 5 cut box | 35 / 84 | 35 / 84 | 36 / 86 |
| 6 fillet | 36 / 87 | 36 / 87 | 37 / 89 |
| 7 hole | **39 / 99** | **39 / 99** | **40 / 101** |

seed 67: every platform builds 10/24, 14/35, 16/41, 19/47, 25/59, 28/65, 32/74 and ends at
**36 / 86** — the counts are the same on all three.

Golden failure messages (stored reference -> built; unchanged text is omitted):

- seed 13 x86_64: `area: 2101.521015 -> 2112.846954; edges: 103 -> 99; faces: 41 -> 39; volume: 2396.939546 -> 2410.783741`.
- seed 13 aarch64 Linux: identical string to x86_64.
- seed 13 Windows ARM64: `edges: 103 -> 101; faces: 41 -> 40` (the area and volume print equal to
  the stored ones at 6 decimals — the difference is topological only).
- seed 67 x86_64 and aarch64 Linux: `edges: 88 -> 86; faces: 37 -> 36`.
- seed 67 Windows ARM64: **passes** only because `tests/robustness/goldens.json` carries a
  `platforms.windows` override of 36/86.

## 2. The stored references: the fork forked between the behaviour change and main's re-record

The fork base is upstream `main`@`b685b730` (2026-10-18T03:12:23-04:00); this branch is that
tree plus the arm64 port docs/CI and the diagnostic commits (whose hook is inert). The shared
references it carries — seed 13 **41/103**, seed 67 **37/88** — are that base's file, not a
measurement of anything this branch builds.

Upstream `main` has moved on and already re-recorded both seeds:

- `72b7fc38` "Fix red main: record robustness seeds 13, 67 and 94 after #2064 (#2396)"
  (2026-10-18T05:38:41-04:00, **not** in this fork): seed 13 41/103 -> 39/99,
  seed 67 37/88 -> 36/86, seed 94 26/61 -> 27/63. Its note:
  > #2064 (multi-profile extrusion) merges coplanar faces the boolean used to leave split. Seed
  > 67 has one face/two edges fewer from its first cut on, at the same volume at every step; seed
  > 94's last box cut leaves one face/two edges more, same volume and area. Seed 13's second
  > cylinder cut leaves 35 faces/84 edges instead of 37/88 at the same volume, so its random
  > fillet (edges picked by index) rounds different edges: 2688.99 instead of 2675.18 mm3 after
  > the fillet; the hole after it removes the same 278.2 mm3. Every step passes the validity and
  > volume-direction checks. Pull request CI ran only every 8th seed, so main was the first run to
  > see these.
  The note names the same feature this report's traces diverge at (seed 13's second cylinder cut)
  and the mechanism: #2064's coplanar-face merging in the prism/boolean path (`coplanar.rs`,
  `prism.rs`, `ops.rs`).
- Later, `a4c4b411` ("Combine: a cut keeps a body joined from separate pieces in one body (#2247)",
  not in this fork) changed seed 67 again, and `f5a868ef` ("Fix red main after #2247: record its
  robustness seeds, and keep older designs' joins of split pieces working (#2540)",
  2026-10-18T13:10:09-04:00) re-recorded seed 67 36/86 -> 35/84 (plus seeds 7, 9, 12, 16, 18, 22,
  27, 31, 33, 45, 50), keeping the seed 67 `platforms.windows` override at 36/86.
- Main today: seed 13 39/99 (no Windows override), seed 67 35/84 shared + 36/86 Windows override.

So what this branch builds is main's recorded behaviour, one file version behind:

- **seed 13**: 39/99 on x86_64 and aarch64 Linux (= #2396's recorded value); 40/101 on Windows
  ARM64, a genuine platform difference that main has no record of either.
- **seed 67**: 36/86 on all three platforms (= #2396's recorded value, = main's Windows override).
  Main's 35/84 is the later #2247 change, which this fork does not contain.
- The fork's CI redness on both seeds is therefore stale references, not a platform disagreement.

**The commit that flipped this branch's builds, on aarch64 Linux**, measured with forced clean
rebuilds (delete `crates/doc/src/format`, re-extract the commit, `cargo clean -p` the
doc/kernel/engine/io/sketch/geom crates before every rebuild):

- Every probed commit from `40abcbc8` through `f4808981` builds seed 13 as 41/103 (including the
  0.15.0 release's neighbourhood — `979614e6` itself was not probed, but `843fa0da` and
  `9be8bb5e`, after it, pass).
- `487aa64e` (#2064) still builds seed 13 as 41/103; `2acc344a` still builds 41/103 and 37/88.
- `83c31d77` (#2129, "surface.boundary: surfaces through boundary curves and curve networks",
  2026-10-17T21:47:38-04:00) is the first failing tree for **both** seeds: seed 13
  41/103 -> 39/99, seed 67 37/88 -> 36/86. The commits between `2acc344a` and `83c31d77` touch
  no crate or `tests/robustness` file (CI and generated files only), so the flip is #2129's merge.
- #2129's merge changes shared numeric code: `crates/kernel/src/surfaces.rs`,
  `crates/kernel/src/boundary.rs`, the new `crates/geom` spline/network modules, `crates/doc`
  evaluation, and the vendored `vendor/truck-geometry/src/nurbs/bspsurface.rs` second-derivative
  fix (`uuder`/`vvder` multiply by p·(p − 1) instead of p).
- The format-pending marker is **not** the mechanism: removing the two pending format files at
  HEAD (format 27 -> 26) leaves seed 13 at 39/99.
- Main's note attributes the behaviour to #2064; this aarch64 measurement puts the crossing at
  #2129. Both are consistent if the coplanar-merge decision #2064 introduced sits at a numeric
  boundary that different commits cross on different architectures (#2064 on main's x86_64 per
  the note; #2129 on aarch64 per measurement). Which of #2129's changes moved aarch64 across,
  and where x86_64 crossed, are not isolated here (§9).

**Build-method caution for anyone repeating this.** `crates/doc/build.rs` generates the format
list from the *listing* of `crates/doc/src/format`, so an extract-over-extract mirror can keep
stale format files and stale generated output. The earlier incremental probes during this
diagnosis — which reported `f4808981`/`4a70c03d` failing, the release `979614e6` as the first
failing commit, and about-to-flip alternation around `b6f9b1de` — were artifacts of that.
Every number in this section comes from the clean-rebuild procedure above.

So "seeds 13 and 67 disagree across platforms" breaks down as:

- **seed 67**: the platforms agree (36/86 everywhere). Only the stored shared value (37/88) is
  stale; main has already re-recorded it (36/86, then 35/84 after #2247).
- **seed 13**: Windows ARM64 genuinely differs from both Linux architectures (40/101 vs 39/99);
  the stored shared value (41/103) matches nothing built at this commit, and main's recorded
  value (39/99) is what both Linux architectures build.

## 3. Step 3: the hook observes, it does not change

- Windows ARM64 and WSL aarch64, seeds 13 and 67, run with `SOLIDCRAFT_TRACE_BOOLEANS` unset and
  then set: after dropping only the harness's own volatile lines (libtest `finished in ...`,
  panic thread ids, cargo progress lines) the outputs are identical character for character. The
  golden failure messages and the debug step tables match, e.g. seed 13 ends
  `vol=2396.939546 area=2101.521015 faces=39 edges=99` both ways on aarch64 Linux.
- Independent reruns of the traced runs produce **identical trace files**: the aarch64 Linux and
  Windows ARM64 traces have the same SHA-256 before and after the workflow-only commit
  (`018138e3` vs `104d6913`).
- The traced CI run's failure strings equal the untraced WSL aarch64 strings (seed 13 and 67).

## 4. Diff 1: x86_64 Linux vs aarch64 Linux

seed 13 (2429 vs 2430 lines):

- First value-only divergence, line 477 — feature 2 `join box`, `planar_face`
  (`crates/kernel/src/heal.rs:552`; the decision is `worst > tol` at `heal.rs:557`):
  - x86_64: `worst` 0x3fe9fddc5350fe8b = 8.12238848426163229e-1
  - aarch64: `worst` 0x3fe9fddc5350fe9b = 8.12238848426165005e-1 (16 ULP)
  - `tol` identical on both: 0x3ec858190a75ce69 = 2.90204669156097466e-6
  - distance to the decision: ≈ 0.81223856 (both stay `keep`; flipping would need a ~2.8e5x change)
- First topology divergence, line 1025 — feature 4 `cut cylinder`, `heal_begin` (emitted at
  `heal.rs:706` from `merge_faces_traced`, called by `heal_prism_result`,
  `crates/kernel/src/boolean_record.rs:190/193`): the prism result before healing has
  **41 faces / 116 edges** (x86_64) vs **42 / 117** (aarch64). Same `branch=faces_only`, same
  `size`/`tol`.
- The last equal line is 1024 — `whole_entry` (`ops.rs:881`): both platforms enter with the same
  28/64/38 body and the same route, differing only in low bits of the measurements
  (`tol_m` 4 ULP; `va` 176 ULP; `vb` identical).
- The counts **converge**: the heal brings both to 35/84 at step 4 and both end at 39/99. Only
  low-bit value differences remain (e.g. step-2 volume 1 ULP; step-3 volume 0x40a76e08d336cc79
  vs 0x40a76e08d336cd27).

seed 67: the two traces are **byte-identical** — 1,227,206 bytes and SHA-256
`0b892f35ee51de0fc4ba83c517fba58ab801ce37c21340e7798ffeb2a993e499` on both.

## 5. Diff 2: Windows ARM64 vs aarch64 Linux

seed 13 (2460 vs 2430 lines):

- First value-only divergence, line 54 — feature 0 `revolved section`, `measure_face`
  (`measure.rs:135`; pick logic `close(f,m)` then `close(m,c)` at `measure.rs:122-133`):
  - Windows: `area_f` 0x4069c537de8203d1 = 2.06163069967208259e2; `area_m` 1 ULP lower than Linux
  - Linux: `area_f` 0x4069c537de8203d0 = 2.06163069967208230e2; `area_c` identical on both
  - margins: `close_fm` = 0.1434 vs threshold 0.01·max(f,m)+1e-9 ≈ 2.0617; `close_mc` = 0.4416
    vs ≈ 2.0602 — both `close` hold on both platforms, branch `fine` on both.
- First topology divergence, line 1025 — the same prism point as Diff 1: **43 / 118** (Windows)
  vs **42 / 117** (aarch64 Linux).
- Unlike Diff 1 it **persists**: step 4 36/86 vs 35/84, step 6 37/89 vs 36/87, end 40/101 vs
  39/99.

seed 67 (both 3601 lines):

- First value-only divergence, line 125 — feature 1 `cut cylinder`, `merge_pair`
  (`heal.rs:166`; decision `dot > 1.0 - 1e-9 && sep < tol` at `heal.rs:140`):
  - Windows: `dot` 0xbcfb91b91b91b91e = −6.12159895424086506e-15; `sep` 0x4029000000000100 = 1.25000000000004547e1
  - aarch64: `dot` 0x0000000000000000 = 0; `sep` 0x40290000000000c8 = 1.25000000000003553e1 (56 ULP)
  - `tol` = 4.85412195973689957e-6 (feature 1 `heal_begin`, size 48.541219597369), so `dot` is
    ~1.0 and `sep` ~12.5 away from flipping; branch `skip`, `keep_ok=1` on both.
- **No topology divergence anywhere** in either file: all faces/edges/vertices and all step
  summaries are identical; both end 36/86.

## 6. Where the first divergent decision is (all pairs)

For every pair, the first difference that changes a discrete decision rather than a low bit is at
the **same** trace point: the face/edge counts of the prism result for seed 13's feature 4
`cut cylinder`. The trace brackets it between line 1024 (`whole_entry`, `ops.rs:881` — identical
inputs 28/64/38 on all three platforms) and line 1025 (`heal_begin` — 41/116 x86_64, 42/117
aarch64 Linux, 43/118 Windows ARM64). Between those points the early prism route builds the
result: `boolean_whole_with_refinement` -> swept-prism/prism boolean (`crate::prism`, **not
instrumented**) -> `prism_result` -> `heal_prism_result` (`boolean_record.rs:190`) ->
`merge_faces_traced`. So the first divergent discrete choice sits inside the untraced prism
boolean/projection for this prism-route cut; the trace shows only its pre-heal counts. No branch
flip in any traced predicate precedes it — all traced threshold decisions agree on all three
platforms.

This is the same point main's #2396 note describes from the behaviour side: the second cylinder
cut "leaves 35 faces/84 edges instead of 37/88", and the decision (whether the prism boolean
leaves coplanar faces split or merges them) lives in `coplanar.rs`/`prism.rs` — none of it is
instrumented, which is why the trace only shows its pre-heal counts.

## 7. Classification

- **x86_64 vs aarch64 Linux, seed 13**: first divergence value-only (16 ULP in a heal predicate,
  branch unchanged, margin ≈ 0.81); the one control-flow difference (prism pre-heal count) does
  not propagate — the final topology is identical (39/99 both).
- **Windows ARM64 vs aarch64 Linux, seed 13**: first divergence value-only (1 ULP tessellation
  area, decision margins > 1.6); the control-flow difference at the prism pre-heal counts
  persists to the final body (40/101 vs 39/99).
- **seed 67**: x86_64 vs aarch64 Linux fully identical (byte-identical traces); Windows vs Linux
  value-only (dot/sep low bits), same topology (36/86) on all three.

## 8. Artifacts

| platform | file | full bytes | committed bytes | full-file SHA-256 | committed SHA-256 |
|---|---|---|---|---|---|
| x86_64 Linux | trace_x86_64_seed13.txt | 931,884 | 931,884 | `6f78e7d97a85b04468fca40935cc0e65032a5598c818d6a9eda821100d3c6e23` | same |
| x86_64 Linux | trace_x86_64_seed67.txt | 1,227,206 | 143,050 (first 400 lines) | `0b892f35ee51de0fc4ba83c517fba58ab801ce37c21340e7798ffeb2a993e499` | `c61c9fcdc28e73dcefaa9276f626ba8902677064a545fc215fea69f3ecad71b6` |
| aarch64 Linux | trace_aarch64_seed13.txt | 932,209 | 932,209 | `c2b2558c1a621f38dfaa5a4a3759d023296e8b8c465b094d27dbe434cb464146` | same |
| aarch64 Linux | trace_aarch64_seed67.txt | 1,227,206 | 143,050 (first 400 lines) | `0b892f35ee51de0fc4ba83c517fba58ab801ce37c21340e7798ffeb2a993e499` | `c61c9fcdc28e73dcefaa9276f626ba8902677064a545fc215fea69f3ecad71b6` |
| aarch64 Windows | trace_aarch64_win_seed13.txt | 945,333 | 945,333 | `93aa8a7c3c213ab2a4a6f60d15a1c6e70cb76ca3955687c476ac79a5f07f53b9` | same |
| aarch64 Windows | trace_aarch64_win_seed67.txt | 1,227,463 | 143,072 (first 400 lines) | `f20e0d64187e3be59b35d8cc3753941b732e54d51541df2aeafd7bb700695dc6` | `2bfc603caf81a2b598ce468b6f4ece95c17b6c1a584255d22fc41cde570d1a4e` |

The seed 67 files exceed the 1 MB commit limit, so what is committed is the first 400 lines per
file (the first divergence, seed 67 line 125, sits well inside); the full files are the CI
artifacts (7-day retention) and are reproducible with the commands in
`docs/ports/traces/round2/README.md`. The full seed 67 files for x86_64 and aarch64 Linux are
byte-identical, so their committed 400-line files are too.

## 9. Untraced and not verified

- `crate::prism` internals (projection and 2D boolean) and `coplanar.rs`: not instrumented; the
  divergent discrete decision is inside there.
- The truck boundary (`truck_call`/`truck_out`) brackets nothing for feature 4 because it takes
  the prism route.
- Windows **x86_64** was not measured; the only records are the `platforms.windows` overrides
  added by #2064 (architecture unstated). This diagnosis measured Windows ARM64 only.
- Main's own CI at this commit's date was not directly observed (its runs were pending/cancelled
  during collection); main has since re-recorded seeds 13/67/94 (#2396) and seed 67 (#2540), and
  this fork's runs at `104d6913` fail both seeds on the stale references.
- Where **x86_64** crossed the coplanar-merge boundary (main's #2396 note says #2064; measured
  here only for aarch64, at #2129) and which of #2129's changes moved aarch64 across the
  threshold (e.g. the vendored `uuder`/`vvder` second-derivative fix) are not isolated.
