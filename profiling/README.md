# Profiling the direct-text gradient fix (#389)

This harness compares the same CPU workload against upstream
`6dd55434177690c845fc5e6f8e9d5a2fe6f1b277` and its single-commit fix,
`38d649c699b339f040a2e8f8a13ed28e1644baf7`. These are the revisions used by
the original gradient repros. `prepare` checks that the fix's parent is the
baseline and that dependency versions match. Both executables compile the
same Rust source and use the same copied Roboto Flex font. Existing image
repros and their lockfiles remain independent.

The renderer is `Void`: this measures shaping, outline handling, tessellation,
paint parameters, command generation, and command disposal. It does not
measure GPU execution, image sampling, or actual shadow-blur shaders. The
original GPU repros remain the visual-correctness check. `black_box` barriers
keep canvas/command-generation work observable to the optimizer.

## Portable timing and allocations

Requirements: Python 3.9+, Git, and a Rust toolchain compatible with the pinned
femtovg revision. No Python packages, platform fonts, window system, or GPU
are needed. Run from the repository root:

```sh
python3 profiling/profile.py prepare
python3 profiling/profile.py run --out out/timing --repeats 7
python3 profiling/profile.py run --out out/stages --measure stages --repeats 7
python3 profiling/profile.py prepare --instrumentation allocations
python3 profiling/profile.py run --build target/profiling/allocations/build.json --out out/allocations
```

`prepare --offline` keeps Cargo offline; Git may still fetch missing revisions.
`--base-source` and `--fixed-source` accept local repositories. A checkout with
local changes is refused. Full revision hashes are the reproducible default.
Override both revisions to compare a newer isolated fix and its parent.
The two committed profiling lockfiles are used with `--locked`. Release
builds retain debug information for profiler symbolication. Native CPU
instructions are not requested by default; any Rust build flags in the
environment are recorded in `build.json`.

On a small machine, `CARGO_BUILD_JOBS=1` limits build memory. To reuse the
plain build's dependency cache for allocation instrumentation, use
`prepare --instrumentation allocations --cargo-target-dir target/profiling/plain/cargo`.
Each mode still publishes separate binary copies and checks their hashes;
finish all builds before measuring so compilation cannot interfere.

| Case | What it exercises |
| --- | --- |
| `gradient-fill`, `gradient-stroke` | 100 px outline text, including cached layout work |
| `glyph-fill`, `glyph-stroke` | Pre-shaped positioned glyphs, excluding per-draw text layout |
| `dashed-stroke` | Outline text through the dash handling added to the helper |
| `image-fill` | Outline text with image paint parameter generation |
| `shadow-fill` | Outline text with varying paint alpha and shadow commands |
| `transformed-fill` | 48 px gradient text under scale 2, which selects outlines |
| `atlas-fill` | 90 px untransformed text, an atlas-path control |
| `path-fill`, `path-stroke` | Ordinary curved paths through the shared helpers |

`--cases`, `--motions`, `--texts`, and `--lines` take comma-separated lists.
Motion is `stationary` or `moving` (bounded quarter-pixel x steps). Text is
`alphabet`, `sentence`, or `diverse` (printable ASCII, rotated per line).
Multiple lines use different origins and, for outline cases, up to sixteen
font sizes. That also causes different last-transform cache entries, even
with stationary text; it is a cache-pressure workload, not a claim that all
outlines fit or miss any particular hardware cache.

```sh
python3 profiling/profile.py run --out out/working-set --cases gradient-fill,glyph-fill --texts diverse --lines 1,16 --frames 1000
```

Each child reports canvas setup and its first frame separately, then warms up
before the measured loop. First-frame cost includes cold layout and outlines
for text cases; pre-shaped cases put initial layout in setup. Font file I/O
is outside setup. Bulk mode reads the clock around the whole loop. Stages mode
times draw and flush separately, writes chronological frame samples, and
reports mean, p50, p95, and p99. Its per-frame clock overhead makes its timing
different from bulk mode, especially for small path/atlas controls.

Allocation accounting is a separate build with an instrumented Rust system
allocator. It reports successful allocation/reallocation calls, requested
bytes, and live requested bytes at the start/end/peak of the measured loop.
A realloc counts its full requested new size. These numbers do not represent
allocator-reserved memory or allocations made directly by native libraries.
Atomic accounting changes execution cost; use the plain build for timing
and hardware-counter comparisons. Peak process RSS is reported separately
on macOS/Linux using per-child `wait4` usage; it includes startup and warmup.
Platforms without that facility leave process resource measurements missing.

On Linux, each child also gets before/after sysfs snapshots of temperatures,
CPU frequency, and governor (the pinned CPU, or all CPUs without `--cpu`).
Temperature readings retain millidegrees Celsius and frequencies retain kHz;
missing or unreadable values are null. Snapshots occur outside the child and
its timing interval. They are endpoints, not a continuous record of thermal
peaks, frequency changes, or throttling during the measured loop.

Each round reverses workload order and alternates baseline/fix order **for
each workload**. Summaries show medians, IQR, and paired percent changes.
IQR describes observed variation, not statistical confidence. Zero-baseline
ratios remain undefined. `summary.json` includes paired stage, allocation,
and memory changes; `summary.md` has the compact timing table. `runs.jsonl`,
per-child output/errors, and stage sample files preserve the raw data.
`experiment.json` records the schedule, commands, host, build provenance,
runner hash, and optional `--notes` about power/thermal conditions.

Use new output directories for experiments. Reusing one is refused to avoid
mixed samples. To regenerate a report:

```sh
python3 profiling/profile.py report out/timing
```

If contention or another problem is discovered after a run, preserve the data
and mark its interpretation explicitly:

```sh
python3 profiling/profile.py mark-validation out/timing --reason 'Concurrent CPU-heavy build battery; timings and cache counters are not performance evidence.'
```

This annotates `experiment.json` and adds a notice to generated reports.
Regenerating timing reports preserves the notice. Raw samples are unchanged.

## M4 / Instruments

Requires full Xcode and permission to profile local processes. Timing and
counter capture are separate experiments. Build with public `os_signpost`
markers around the measured loop:

```sh
python3 profiling/profile.py prepare --instrumentation signposts
python3 profiling/profile.py trace --build target/profiling/signposts/build.json --out out/counters --seconds 8 --repeats 3
python3 profiling/profile.py trace --build target/profiling/signposts/build.json --out out/processing --counter-mode processing --seconds 8 --repeats 3
python3 profiling/profile.py trace --build target/profiling/signposts/build.json --out out/efficiency --qos background --counter-mode processing --seconds 8 --repeats 3
```

The signpost build creates its log at startup, emits an unmeasured readiness
event, and waits one second before setup and warmup. On this Xcode installation,
creating the log immediately before measurement sometimes captured only End;
initializing it earlier alone did not resolve that. Before Begin, the harness
also checks that logging is enabled, waiting up to five seconds and failing if
it stays disabled. These waits are outside the measured interval. The exporter
still requires exactly one complete target-process interval; an enabled log
alone does not prove Instruments captured both events.

`background` QoS encourages efficiency-core placement; it does not pin a
core. Read the **observed** core residency in `counter-runs.json` before
describing results as efficiency-core measurements. `--case`, `--motion`,
`--text`, and `--lines` select one workload per trace experiment. The loop
runs 100-frame batches for at least `--seconds`; frame counts can differ,
so counts are normalized per measured frame.

The collector discovers and saves the installed template's recording options.
Guided `--counter-mode` choices are `bottlenecks`, `processing`, and `delivery`.
For custom event/formula selection use `--recording-options FILE` with a
configuration exported by this Xcode, or `--template FILE.tracetemplate`.
Event availability and semantics vary by CPU and Xcode; a generic cache-miss
counter is not automatically an L1D miss or a prefetch miss.

Each trace gets a table of contents, selected raw XML tables, resolved named
metric samples, and a measured-interval summary. XML references are resolved
using schema column names. Summaries select the target process and signposted
thread, exclude startup/warmup, and discard sample intervals crossing either
measurement boundary. Counts are summed; reported ratios are duration-weighted
means, not event counts. `counter-summary.md` separates observed core types.
Raw counter arrays are retained without inventing meanings for their indices.
Open `.trace` bundles in Instruments for source/assembly attribution and
sampling modes such as L1D Cache Miss Sampling.

For short call/instruction investigations on supported hardware:

```sh
python3 profiling/profile.py trace --build target/profiling/signposts/build.json --out out/processor-trace --template 'Processor Trace' --seconds 0.05
```

Processor Trace may require enabling it in macOS Developer Tools settings.
Keep recordings short because its instruction streams can be large. The
collector preserves the bundle; it does not claim to calculate hardware
return-address-stack occupancy from call depth. Matt's phrase “peak RAS”
needs clarification before claiming coverage of that metric.

## Linux / Raspberry Pi

The same plain executables work with Linux `perf` when PMU access is enabled:

```sh
python3 profiling/profile.py prepare
python3 profiling/profile.py run --out out/perf --collector perf --cpu 0 --frames 30000 --repeats 7
```

`--cpu` applies Linux CPU affinity to the child, inherited by `perf`'s workload.
The collector saves `perf list` and the version. Use `--events` to select
events supported by the device/kernel, preferably in small groups to avoid
multiplexing. Parsed events preserve running percentage, original runtime,
units, and raw lines. Unsupported events remain missing, not zero. The Linux
collector counts the **whole process**, including startup and warmup; its
normalized fields say `process_event_*_per_measured_frame`. Increase measured
frames to amortize those costs, and do not equate them with macOS's signposted
interval counts. Collection has been validated on a physical Pi 3B running
32-bit Linux; results and limitations are below.

Avoid simultaneous compilation or profiling during timing experiments.
Keep power and thermal conditions consistent and record them with `--notes`.
An M4 result characterizes this machine and workload; it does not establish
performance on Pi 3B/4/Zero or iPhone.

## Validation

```sh
python3 -m unittest discover -s profiling -p 'test_*.py'
python3 profiling/profile.py run --out out/smoke --frames 50 --warmup 5 --repeats 2 --measure stages
```

The tests cover run pairing, missing counter values, incompatible work counts,
artifact integrity, XML references, target-process selection, and measurement
boundaries. The smoke run executes every case against both revisions.

## Initial M4 Max rerun (2026-10-08)

The first datasets (`out/m4-389-*` without `idle`) are marked collection
validation only after the user reported concurrent CPU-heavy work. A subsequent
sequential rerun followed the user's report that this work had finished.
Power and thermal conditions were not controlled. The local artifacts are:

- [Bulk timing](../out/m4-389-idle-timing/summary.md): 11 cases, two motions,
  seven paired rounds, 3,000 measured frames per child.
- [Draw/flush stages](../out/m4-389-idle-stages/summary.md): five cases, two
  motions, five paired rounds; 300,000 chronological frame samples.
- [Allocations](../out/m4-389-idle-allocations/summary.json): all 66 paired runs
  matched exactly for allocation calls, requested bytes, and live/peak bytes.
- [Efficiency-core processing counters](../out/m4-389-idle-processing-e-final/counter-summary.md):
  stationary gradient fill, three paired eight-second intervals. All sampled
  residency was on efficiency cores. Raw per-recording residency, durations,
  and metrics are in `counter-runs.json` beside the report.

Median paired bulk gradient fill changes were -0.52% stationary and -1.37%
moving; their change IQRs crossed zero. Gradient stroke changes were -1.94%
stationary and -0.63% moving. These are initial observations from this run,
not confidence bounds or evidence for other machines.

Median sampled cycles per frame were 229,107 baseline and 228,994 fixed.
Instruments' Critical L1D Cache Miss ratio was 1.24% baseline and 1.50% fixed;
individual recordings ranged 1.18–1.36% baseline and 1.16–1.68% fixed. This is
a reported ratio, not an L1D miss count or miss rate. The three pairs do not
establish a consistent cache benefit or regression. Branch/prefetch events
and Processor Trace need separate experiments; this processing-mode run
does not cover them.

Three earlier idle counter attempts lacked a complete signpost interval and
were rejected. A two-second paired smoke capture and all six subsequent
eight-second captures passed after adding the startup handoff.

## Raspberry Pi 3B / ARM32 run (2026-10-09)

The [shareable Pi reports and raw-data archive](results/pi3-2026-10-09/README.md)
preserve the nine datasets without the compiled binaries.

This is a rented physical Raspberry Pi 3 Model B Rev 1.2, CPU part `0xd03`
(Cortex-A53). Its Raspbian 12 installation uses a 32-bit `armv7l` kernel
6.1.21-v7+, so Rust 1.96.0 / LLVM 22.1.2 built for
`armv7-unknown-linux-gnueabihf`. It is not an AArch64 result. Linux `perf`
6.12.109 exported a PMU named `armv7_cortex_a7`; the reports retain its generic
event names rather than infer raw event encodings from that name.

Builds ran with one Cargo job and shared dependency artifacts between plain
and allocation modes. Both revisions, dependency versions, copied font,
and binary hashes were checked. All compilation and result transfers
finished before each measurement batch. Workloads ran sequentially, pinned
to CPU 2 with the performance governor; measurement endpoints reported
1,200,000 kHz. Original `ondemand` governors were restored after each batch.
The device clock was synchronized by NTP before measurement.

The first batch had 1,000 child runs, including 88 collection-validation
smoke children. The two longer follow-ups add 152 children, for 1,152 total.
Local artifacts are under `out/pi3-armhf`:

- [Bulk timing](../out/pi3-armhf/pi3-389-timing/summary.md): all 11 cases,
  two motions, nine paired rounds, 500 measured / 100 warmup frames.
- [Draw/flush stages](../out/pi3-armhf/pi3-389-stages/summary.json): four
  outline cases, two motions, five paired rounds, 500 measured frames.
  All 40,000 frame samples have matching draw/flush sums. The smoke adds
  another 4,400 validated samples.
- [Allocations](../out/pi3-armhf/pi3-389-allocations/summary.json): all 66
  pairs match exactly for calls, requested bytes, and live/peak bytes.
- [Core counters](../out/pi3-armhf/pi3-389-perf-core/summary.json) and
  [cache counters](../out/pi3-armhf/pi3-389-perf-cache/summary.json): separate
  batches, four outline cases, two motions, seven paired rounds, 2,000
  measured / 100 warmup frames. Supported events reported 100% running time
  in every child; no reported multiplexing. `L1-dcache-prefetch-misses` is
  unsupported in all 112 cache children and remains null.
- [Working-set timing](../out/pi3-armhf/pi3-389-working-set/summary.md):
  diverse ASCII text, one/eight lines, two motions, five paired rounds,
  100 measured / 100 warmup frames. Eight lines varied font sizes/origins
  and drew 752 glyphs per frame.
- [Longer plain timing](../out/pi3-armhf/pi3-389-timing-long/summary.md):
  four outline cases, seven paired rounds, 2,000 measured / 100 warmup frames.
- [Longer diverse-text timing](../out/pi3-armhf/pi3-389-diverse-long/summary.md):
  one line, fill-text and pre-shaped glyph cases, five paired rounds,
  1,000 measured / 100 warmup frames.

Stage measurements put most of the measured frame in draw work: median
flush means for gradient cases were 7.3–8.4 microseconds, versus draw means
of 568–1,301 microseconds. Some moving runs had millisecond-scale p99 tails;
frame distributions and chronological samples remain available for inspection.

The short bulk runs show large outliers, including in atlas/path controls.
The longer plain runs check the outline timing independently of `perf` and
allocation instrumentation. Median paired changes from those runs are:

| Text / workload / motion | Base us/frame | Fixed us/frame | Paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| sentence / glyph-fill / moving | 1264.22 | 1254.27 | -0.95% | -1.50% to -0.33% |
| sentence / glyph-fill / stationary | 985.19 | 978.00 | -0.84% | -1.13% to -0.23% |
| sentence / glyph-stroke / moving | 954.63 | 935.21 | -1.41% | -2.68% to -0.65% |
| sentence / glyph-stroke / stationary | 585.89 | 575.70 | -2.20% | -2.37% to -1.71% |
| sentence / gradient-fill / moving | 1276.33 | 1264.15 | -0.87% | -1.56% to -0.60% |
| sentence / gradient-fill / stationary | 1008.84 | 987.68 | -2.10% | -2.40% to -0.97% |
| sentence / gradient-stroke / moving | 963.87 | 956.38 | -0.62% | -2.32% to -0.04% |
| sentence / gradient-stroke / stationary | 598.29 | 588.76 | -1.53% | -1.77% to -1.14% |
| diverse / glyph-fill / moving | 4635.91 | 4614.15 | -0.25% | -0.67% to -0.10% |
| diverse / glyph-fill / stationary | 3066.81 | 3035.76 | -0.49% | -1.07% to -0.33% |
| diverse / gradient-fill / moving | 4688.40 | 4618.15 | -1.64% | -1.85% to -1.03% |
| diverse / gradient-fill / stationary | 3112.15 | 3069.40 | -1.34% | -1.56% to -1.14% |

The original eight-line diverse-text experiment had median changes between
-0.62% and +0.90% across its four case/motion combinations. It does not
establish whether the working set fits any particular hardware cache.

The following are median paired changes in whole-process event counts per
measured frame for **sentence** text. Core and cache columns come from
separate experiments. Startup and warmup remain included; these are neither
isolated measured-loop counts nor miss rates. IQRs and individual pairs are
in the linked JSON reports; these point summaries are not confidence bounds.

| Workload / motion | Cycles | Instructions | Branch misses | L1D load misses | L1I load misses |
| --- | ---: | ---: | ---: | ---: | ---: |
| glyph-fill / moving | -1.21% | -0.44% | +0.06% | +8.77% | -8.84% |
| glyph-fill / stationary | -1.09% | -0.56% | -0.51% | +5.82% | +1.65% |
| glyph-stroke / moving | -1.51% | -0.57% | -0.72% | +2.28% | -12.76% |
| glyph-stroke / stationary | -1.91% | -0.91% | -2.03% | -5.65% | -8.04% |
| gradient-fill / moving | -0.64% | -0.31% | +0.40% | -5.69% | +3.50% |
| gradient-fill / stationary | -0.96% | -0.55% | -0.01% | -5.12% | -6.97% |
| gradient-stroke / moving | -1.81% | -0.57% | -1.39% | -2.59% | -12.88% |
| gradient-stroke / stationary | -2.22% | -0.90% | -2.45% | -4.60% | -8.31% |

The sentence experiments show fewer cycles/instructions in their medians,
but not a uniform reduction in cache or branch misses. For example,
pre-shaped fill glyphs have increased L1D load-miss counts in every cache
pair, while gradient-fill medians decrease. dTLB counts vary substantially
between runs (some medians are below one event per measured frame), so their
large percentage swings need absolute-count context. This data does not
justify claiming that no low-level metric regresses.

The 100-frame diverse-text run suggested +9.10% moving gradient fill and
+10.78% moving pre-shaped glyph fill. In the longer plain follow-up the
medians were -1.64% and -0.25%, respectively. The earlier slowdown did not
persist in those longer averages. Motion repeats every 128 frames: the
short run covers a partial cycle and the longer run nearly eight cycles,
so these runs weight positions differently and do not rule out a
position-specific regression. Future moving-workload comparisons should
use measured-frame counts divisible by 128.

Recorded measurement endpoint temperatures were 40.8–49.9°C.
`vcgencmd get_throttled` was `0x0` before and after both batches. Full conditions/restoration
snapshots are in [pi3-system.json](../out/pi3-armhf/pi3-system.json) and
[pi3-system-followup.json](../out/pi3-armhf/pi3-system-followup.json).
Endpoint telemetry is not a continuous temperature trace. The
[provenance directory](../out/pi3-armhf/pi3-provenance) preserves the exact
plain/allocation binaries, font, build metadata, profiling sources, system
information, and detailed `perf list`; copied binary/font hashes match the
build records. Raw data and compiled artifacts are ignored by Git.

This remains CPU work through `Void`, without GPU rendering. Prefetch misses,
hardware RAS behavior, AArch64, Pi 4/Zero, and iPhone are not covered by this run.
