# Pi 3 best-improvement search for #389

The largest independently confirmed median change was **-17.94%** for `offscreen-fill-batched` (7/7 pairs faster; paired IQR -20.09% to -17.87%). This is the strongest result among these candidates, not a bound on the maximum possible improvement.

This is CPU command generation with `Canvas<Void>`; no GPU rendering or end-to-end application speedup is measured. Overlap layouts repeatedly draw the same glyph at the same location and deliberately favor a hot transformed-outline cache. Spaced grid/digit-label layouts are separate practical companions. A fully off-screen overlap workload is a synthetic CPU-overhead case.

## Hypotheses

| Hypothesis | Why it could favor the PR | Tested scenes |
|---|---|---|
| Cheap cached outline fills | With a straight `I` outline and fixed position, state copies, transform updates and restoration become a larger fraction of remaining work. | Visible overlap; AA on/off; grid companion |
| Cheap outline strokes | A simple cached stroke exposes the same removed per-glyph state work through the stroke path. | Visible batched stroke |
| Pre-shaped batches | One glyph-run call amortizes shared font/context setup while preserving the per-glyph work removed by the PR. | Matched batched/unbatched visible fill |
| Mostly off-screen text | Bounds rejection avoids geometry expansion and parameter setup, leaving the removed state operations more visible. | 0% and 10% visible overlap; 10% visible spaced grid |
| Small rotated labels | Rotation forces outline rendering even at 18 px; smaller outlines can reduce common tessellation work. | Ten-digit labels, 0.12 rad rotation, fill/stroke; identity atlas control |

The Pi's in-order Cortex-A53 and small L1 caches motivate looking for workloads where repeated state writes and matrix operations matter. This is a hypothesis about the code path, not proof that a particular cache or pipeline effect caused a timing change. See the [Arm Cortex-A53 TRM](https://documentation-service.arm.com/static/6040c321ee937942ba301626).

## Hardware and comparison

| Item | Configuration |
|---|---|
| Board | Raspberry Pi 3 Model B Rev 1.2, revision a02082 |
| CPU | Cortex-A53 r0p4, 4 cores; measured children pinned to CPU 2 |
| Frequency | Performance governor; 1.2 GHz checked before/after each measured child |
| Cache | 32 KiB L1I + 32 KiB L1D per core; shared 512 KiB L2 |
| Memory | Nominal 1 GiB; approximately 922 MiB usable; no swap |
| OS | Raspbian GNU/Linux 12 Bookworm, 32-bit armhf |
| Kernel | Linux 6.1.21-v7+, armv7l |
| Compiler | Rust 1.96.0, LLVM 22.1.2; default release target flags |
| Font | Bundled Roboto Flex, identical bytes in both variants |
| Master | `6dd55434177690c845fc5e6f8e9d5a2fe6f1b277` |
| PR | `38d649c699b339f040a2e8f8a13ed28e1644baf7`, one commit on master |

All new scenes submit 512 glyphs per frame, prepared outside timing. Fill/stroke paints are solid. Font size is 100 px except the 18 px labels. AA is off except the explicit AA fill, label scenes, and atlas control. Motion is stationary. The canvas is 16384 × 16384; off-screen placement is beyond its right edge. Visibility percentages describe prepared placement; the rotated label tests are 100% visible.

## Selection and independent confirmation

Six existing-binary configurations first received three short paired rounds at a 0.25 s measured target. Their noisy results motivated testing the more specific mechanisms rather than promoting an apparent single short-sample win.

The new first pass used 11 configurations, three paired rounds, 0.30 s measured targets and 16 warmup frames. Frame counts are calibrated on master and shared by both variants, rounded up to whole 16-frame cycles. Each round alternates master/PR order, and scenario order reverses between rounds. The promotion rule selected at most one candidate per family (visible overlap, off-screen overlap, spaced scenes), at most three overall, with median change at most −1% (at least two of three pairs faster). The atlas control cannot be promoted. Full ranked selection decisions are in [selection.json](selection.json).

Promoted candidates received seven fresh paired rounds at a 1.0 s measured target. Selection samples are excluded from their confirmation statistics. Builds completed before timing, and archives/downloads began after all timing and counters finished. The governor was restored afterward; endpoint firmware checks reported no throttling. Endpoint checks are not continuous monitoring.

Absolute columns are separate variant medians. Changes and IQRs are computed within master/PR pairs, so the percentage need not equal a ratio of the separate medians. Negative changes mean less time. Positive numbers have no leading plus sign.

| Scene | Master ms/frame | PR ms/frame | Paired change | Paired IQR | Faster pairs |
|---|---:|---:|---:|---:|---:|
| `offscreen-fill-batched` | 0.7118 | 0.5841 | -17.94% | -20.09% to -17.87% | 7/7 |
| `visible-stroke-batched` | 2.7771 | 2.5954 | -6.22% | -7.90% to -3.96% | 7/7 |
| `visible-fill-grid` | 5.6510 | 5.5176 | -2.80% | -3.05% to -1.44% | 7/7 |

## Full first pass

These short measurements selected candidates; they do not establish a reliable speedup by themselves.

| Scene | Master ms/frame | PR ms/frame | Paired change | Paired IQR | Faster pairs |
|---|---:|---:|---:|---:|---:|
| `offscreen-fill-batched` | 0.6798 | 0.5527 | -18.69% | -24.15% to -18.01% | 3/3 |
| `mostly-offscreen-fill-batched` | 1.1124 | 0.9370 | -14.86% | -17.00% to -13.31% | 3/3 |
| `visible-stroke-batched` | 2.9162 | 2.4898 | -12.48% | -15.50% to -10.55% | 3/3 |
| `visible-fill-grid` | 5.3253 | 5.2301 | -2.31% | -2.87% to 4.04% | 2/3 |
| `rotated-labels-stroke` | 16.6559 | 16.1696 | -0.47% | -1.69% to 1.99% | 2/3 |
| `visible-fill-batched-aa` | 4.4925 | 4.3499 | 0.37% | -6.97% to 3.68% | 1/3 |
| `visible-fill-unbatched` | 10.6132 | 10.5565 | 0.41% | -6.29% to 7.29% | 1/3 |
| `mostly-offscreen-fill-grid` | 2.5382 | 2.5551 | 1.05% | -3.72% to 1.54% | 1/3 |
| `visible-fill-batched` | 4.3680 | 4.4068 | 1.24% | -8.14% to 5.73% | 1/3 |
| `small-labels-atlas-control` | 0.4834 | 0.5281 | 9.38% | 4.47% to 12.25% | 1/3 |
| `rotated-labels-fill` | 19.6487 | 21.5327 | 9.59% | 3.77% to 11.91% | 1/3 |

### Existing-binary scout

The scout used the previous stress binaries and a separate source snapshot; it is retained in full.

| Scene | Master ms/frame | PR ms/frame | Paired change | Paired IQR | Faster pairs |
|---|---:|---:|---:|---:|---:|
| `mixed-fill-false` | 12.3075 | 10.4233 | -15.31% | -15.43% to -0.86% | 2/3 |
| `cache-one-stationary` | 12.6446 | 11.6625 | -7.77% | -11.38% to -3.48% | 2/3 |
| `cache-one-moving` | 11.9848 | 11.7589 | -2.52% | -4.47% to 3.33% | 2/3 |
| `mixed-stroke-false` | 8.8494 | 8.6428 | -2.33% | -7.00% to -0.23% | 2/3 |
| `mixed-stroke-true` | 8.7348 | 8.6494 | -2.18% | -6.89% to 2.84% | 2/3 |
| `mixed-fill-true` | 10.9591 | 12.1530 | 11.56% | -1.67% to 12.14% | 1/3 |

## Core PMU counts

Only the two strongest confirmed candidates with an entirely negative timing IQR received counters: three additional paired rounds at a 0.60 s target. Counts cover the **whole child process**, including setup, first frame and warmup, then are divided by measured frames. They are not event counts isolated to the steady-state loop. All four events fit in one group without multiplexing (100% running time).

| Scene | Event | Master counts/measured frame | Paired count change | Paired IQR |
|---|---|---:|---:|---:|
| `offscreen-fill-batched` | cycles | 870904.51 | -21.61% | -21.64% to -19.78% |
| `offscreen-fill-batched` | instructions | 684563.64 | -17.21% | -17.22% to -15.79% |
| `offscreen-fill-batched` | branches | 48080.56 | -32.50% | -32.53% to -30.64% |
| `offscreen-fill-batched` | branch-misses | 4708.85 | -50.28% | -50.31% to -48.21% |
| `visible-stroke-batched` | cycles | 3607504.90 | -6.74% | -7.30% to -3.01% |
| `visible-stroke-batched` | instructions | 2441437.91 | -4.27% | -4.51% to -2.73% |
| `visible-stroke-batched` | branches | 220469.83 | -6.52% | -6.82% to -4.72% |
| `visible-stroke-batched` | branch-misses | 24125.83 | -8.28% | -8.65% to -5.97% |

## Code and raw data

This work lives in the [negative repro repository](https://github.com/jfarmer/femtovg-negative-x-repro). The portable harness is in [stress.rs](../../stress.rs), the runner in [stress.py](../../stress.py), and the exact used sources are frozen in [sources-best](sources-best) and [sources-existing](sources-existing). The experiment driver is [pi_best_job.py](pi_best_job.py). Linux affinity/perf collection is optional; the Rust workloads have no platform system calls or GPU/window dependency.

Every phase contains `experiment.json`, `runs.jsonl`, each child's stdout/stderr and summaries; counter phases additionally contain raw perf CSVs. [raw-results.tar.gz](raw-results.tar.gz) contains the complete remote result directory, including source snapshots and selection decisions. Its SHA-256 is `d43f4d490970a76b6f3972f4960fece8aadc771535917e567aa2938c65f6acbe`. [verify_stress.py](../../verify_stress.py) validates equivalent work, source hashes, exact pairing and available measurement conditions.
