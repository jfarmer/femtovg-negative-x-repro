# Pi 3B results for #389 (2026-10-09)

These results compare upstream `6dd55434177690c845fc5e6f8e9d5a2fe6f1b277`
with its single-commit fix, `38d649c699b339f040a2e8f8a13ed28e1644baf7`,
using the [profiling harness](../../README.md).

- Board: physical Raspberry Pi 3 Model B Rev 1.2, Cortex-A53.
- OS: Raspbian 12 Bookworm, 32-bit ARM; kernel 6.1.21-v7+ (`armv7l`).
- Toolchain: Rust 1.96.0, LLVM 22.1.2, `armv7-unknown-linux-gnueabihf`.
- Collector: Linux perf 6.12.109; exported PMU name `armv7_cortex_a7`.
- Measurements: sequential, pinned to CPU 2, performance governor, 1.2 GHz
  at recorded endpoints; original ondemand governor restored after each batch.
- Endpoint temperatures: 40.8–49.9°C; throttling flags `0x0` before/after.

There are 1,152 child runs, including 88 collection-validation smoke runs.
Timing, stages, allocations, core counters, and cache counters use separate
experiments. The renderer is `Void`, so these results cover CPU work only.
Counter counts cover the whole process, including startup/warmup; their
per-frame normalization divides by measured frames. Supported events reported
100% running time. Prefetch misses are unsupported and remain null. No
hardware RAS measurement was made.

## Reports

- [pi3-389-allocations](./pi3-389-allocations/summary.md): [full statistics](./pi3-389-allocations/summary.json), [commands and provenance](./pi3-389-allocations/experiment.json).
- [pi3-389-diverse-long](./pi3-389-diverse-long/summary.md): [full statistics](./pi3-389-diverse-long/summary.json), [commands and provenance](./pi3-389-diverse-long/experiment.json).
- [pi3-389-perf-cache](./pi3-389-perf-cache/summary.md): [full statistics](./pi3-389-perf-cache/summary.json), [commands and provenance](./pi3-389-perf-cache/experiment.json).
- [pi3-389-perf-core](./pi3-389-perf-core/summary.md): [full statistics](./pi3-389-perf-core/summary.json), [commands and provenance](./pi3-389-perf-core/experiment.json).
- [pi3-389-smoke](./pi3-389-smoke/summary.md): [full statistics](./pi3-389-smoke/summary.json), [commands and provenance](./pi3-389-smoke/experiment.json).
- [pi3-389-stages](./pi3-389-stages/summary.md): [full statistics](./pi3-389-stages/summary.json), [commands and provenance](./pi3-389-stages/experiment.json).
- [pi3-389-timing](./pi3-389-timing/summary.md): [full statistics](./pi3-389-timing/summary.json), [commands and provenance](./pi3-389-timing/experiment.json).
- [pi3-389-timing-long](./pi3-389-timing-long/summary.md): [full statistics](./pi3-389-timing-long/summary.json), [commands and provenance](./pi3-389-timing-long/experiment.json).
- [pi3-389-working-set](./pi3-389-working-set/summary.md): [full statistics](./pi3-389-working-set/summary.json), [commands and provenance](./pi3-389-working-set/experiment.json).

The short runs showed large timing outliers. The longer plain sentence
runs had median paired changes from -0.62% to -2.20%. Pre-shaped glyph
fill L1D misses increased in every cache pair, with medians of 5.82%
stationary and 8.77% moving. These are observed paired changes, not
confidence bounds.

The 100-frame diverse-text run suggested 9–11% moving-text slowdowns that
did not persist in the longer averages. Motion has a 128-frame cycle, so
those runs weight positions differently and do not rule out a position-specific
difference. Future moving comparisons should use multiples of 128 frames.

All 66 allocation pairs matched exactly. All 44,400 chronological stage
samples, including smoke, have matching draw/flush sums.

## Raw data

Download [raw-results.tar.gz](./raw-results.tar.gz) and verify it against
[SHA256SUMS](./SHA256SUMS). Extract with `tar -xzf raw-results.tar.gz`.
The archive preserves each child's output/errors, runs.jsonl, perf CSVs,
frame samples, reports, experiment metadata, system/condition snapshots,
and build metadata with binary/font/source hashes. Paths in recorded
commands refer to the machine where the experiments ran.

Compiled executables and the copied font are not included. `prepare` rebuilds
from pinned revisions and locked dependencies. Full compiled artifacts remain
in the local collection under `out/pi3-armhf/pi3-provenance`.
