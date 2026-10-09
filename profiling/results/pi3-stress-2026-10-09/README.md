# Pi 3 adversarial CPU tests for #389

No stable large timing regression emerged from this search. The largest exploratory paired median was 10.73% slower; that rectangle-stroke case became 0.04% in nine fresh pairs. The clearest downside was higher branch-miss counts for tiny ordinary rectangle paths: 4.03% for fills and 2.48% for strokes, positive in all five counter pairs.

This is an observed search over specified workloads, not a bound on every workload or compilation context. All trials, including discarded collection-validation data, are retained.

## Hardware and revisions

| Item | Details |
| --- | --- |
| Board | Raspberry Pi 3 Model B Rev 1.2 |
| CPU | Four Cortex-A53 r0p4 cores; measured child pinned to CPU 2 |
| Frequency | Performance governor, 1.2 GHz at all child measurement endpoints |
| Caches | 32 KiB L1 instruction and 32 KiB L1 data per core; 512 KiB shared L2; 64-byte lines |
| OS | Raspbian GNU/Linux 12 (Bookworm), 32-bit armhf; kernel 6.1.21-v7+ #1642 |
| Compiler | Rust 1.96.0, LLVM 22.1.2; armv7-unknown-linux-gnueabihf; default release CPU flags |
| Renderer | Void: CPU command generation/disposal, with no GPU execution |
| Baseline | `6dd55434177690c845fc5e6f8e9d5a2fe6f1b277` |
| PR | `38d649c699b339f040a2e8f8a13ed28e1644baf7`, one commit on the baseline |

The firmware reported `throttled=0x0` at all 19 phase endpoints. Every child endpoint reported the performance governor and 1,200,000 kHz. These are endpoint readings, not a continuous temperature/frequency trace. The original ondemand governor was restored and all experiment jobs exited. [Hardware](hardware.json) and [telemetry](telemetry.jsonl) are saved.

## Search and independent validation

- Tiny persistent paths: triangles/rectangles and stroked lines; antialiasing on/off; 64, 512, 4096 draws.
- Cached glyph fills: 1, 4, 16, 64, 256, 1024 representations; grouped/shuffled schedules; stationary/moving placement; 2048 draws.
- Mixed glyph/path draws: fills/strokes; 10/50/90% glyphs; solid/gradient; grouped/shuffled schedules; 1024 draws.

The [78-scenario plan](sweep-plan.json) ran three paired rounds (468 children). The [selection record](selection.json) chose the six largest exploratory medians, a candidate from each case, and grouped/shuffled controls. Ten unique configurations then ran nine new paired rounds (180 children). Five case representatives received separate core-counter, cache-counter, stage, and allocation phases.

The largest nine-pair median from each of the three families was then independently repeated with ten pairs and approximately three seconds of measured work per child. The source-v2 harness adds an explicit pre-flush `black_box` barrier to the stress branch, matching the original workloads. Both binaries were rebuilt before that phase. Source-v1 retained the shared drawing and `Params::new` work in the optimized binaries; its saved symbols/disassembly and both exact source snapshots are in the archive. The versions are reported separately.

An earlier source-v2 long-validation attempt overlapped artifact archiving/download. That entire dataset is explicitly [collection validation only](long-validation/summary.md). The final [long-clean](long-clean/summary.md) experiment ran after packing, downloads, and compilation stopped. No pairs were removed from either dataset.

### Final clean timing: source v2, ten independent pairs

| Selected configuration | Master ms/frame | PR ms/frame | Median paired change | Paired IQR | PR slower pairs |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2048 glyph fills; 64 representations; grouped; moving | 97.623 | 96.921 | -0.56% | -0.93% to -0.45% | 0/10 |
| 1024 mixed fills; 50% glyphs; solid; shuffled | 17.559 | 17.411 | -0.91% | -1.22% to -0.62% | 1/10 |
| 512 rectangle fills; AA on | 4.482 | 4.479 | -0.91% | -1.32% to 0.76% | 3/10 |

Positive changes mean the PR did more work/took longer. Times are separate variant medians; changes are medians of within-round ratios, so they need not equal the ratio of the two displayed medians. The tiny-path IQR crosses zero; its largest individual paired slowdown was 3.69%, and its paired median was −0.91%. A single worst pair does not establish a persistent regression.

### Source-v1 candidate timing: nine fresh pairs

| Configuration | Master ms/frame | PR ms/frame | Median paired change | Paired IQR | PR slower pairs |
| --- | ---: | ---: | ---: | ---: | ---: |
| 512 rectangle fills; AA on | 4.462 | 4.518 | 1.56% | -1.27% to 1.95% | 5/9 |
| 64 rectangle strokes; AA off | 0.280 | 0.279 | 1.19% | -2.96% to 2.34% | 5/9 |
| 512 rectangle strokes; AA off | 2.438 | 2.426 | 0.04% | -1.16% to 0.40% | 5/9 |
| 1024 mixed fills; 50% glyphs; solid; shuffled | 17.757 | 17.596 | -0.07% | -3.25% to 0.92% | 4/9 |
| 2048 glyph fills; 64 representations; grouped; moving | 96.541 | 96.777 | -0.34% | -1.38% to 1.13% | 4/9 |
| 1024 mixed strokes; 10% glyphs; gradient; shuffled | 8.201 | 8.146 | -0.69% | -1.56% to -0.55% | 0/9 |
| 2048 glyph fills; 64 representations; shuffled; moving | 94.617 | 93.413 | -1.16% | -2.02% to -0.69% | 0/9 |
| 1024 mixed strokes; 10% glyphs; gradient; grouped | 8.079 | 7.922 | -1.94% | -3.52% to 1.66% | 4/9 |
| 1024 mixed fills; 50% glyphs; solid; grouped | 17.442 | 16.855 | -4.19% | -4.49% to -2.32% | 2/9 |
| 64 rectangle fills; AA off | 0.521 | 0.481 | -6.64% | -10.61% to 8.70% | 3/9 |

## Master absolute counts, then relative changes: source v1

Core and cache counters were separate five-pair experiments on the plain binaries. Each event ran at 100% reported runtime; no multiplexed or unsupported event was used. Counts cover the whole child, including setup, first frame, and 16 warmup frames, divided by measured frames. They are not isolated measured-loop counts. The moving-glyph representative had only 16 measured frames in these counter phases, so warmup is a substantial part of its counts. Bulk timings below come from the separate nine-pair timing phase.

| Configuration | Time ms/frame | Cycles/frame | Instructions/frame | Branches/frame | Branch misses/frame | L1D loads/frame | L1D misses/frame | L1I misses/frame | dTLB misses/frame |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2048 glyph fills; 64 representations; grouped; moving | 96.541 | 238,027,960.4 | 144,178,143.1 | 12,176,648.6 | 2,050,174.8 | 78,456,398.5 | 1,384,187.4 | 2,812,076.7 | 10,536.6 |
| 1024 mixed fills; 50% glyphs; solid; shuffled | 17.757 | 24,656,780.3 | 13,928,050.3 | 1,350,446.3 | 223,072.3 | 7,918,190.0 | 123,025.3 | 423,465.4 | 546.3 |
| 1024 mixed strokes; 10% glyphs; gradient; shuffled | 8.201 | 10,923,452.2 | 5,768,610.9 | 546,221.8 | 68,469.4 | 3,237,961.5 | 51,403.4 | 117,821.0 | 529.9 |
| 512 rectangle fills; AA on | 4.462 | 5,606,304.6 | 3,391,540.2 | 341,671.6 | 48,196.6 | 1,665,387.0 | 25,557.3 | 38,836.8 | 33.7 |
| 64 rectangle strokes; AA off | 0.280 | 326,732.3 | 220,361.3 | 21,296.0 | 2,493.7 | 122,445.6 | 1,729.8 | 1,627.4 | 0.1 |

| Configuration | Time | Cycles | Instructions | Branches | Branch misses | L1D loads | L1D misses | L1I misses | dTLB misses |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2048 glyph fills; 64 representations; grouped; moving | -0.34% | -1.15% | -0.61% | -0.85% | -5.37% | -1.02% | -2.64% | -10.60% | 10.86% |
| 1024 mixed fills; 50% glyphs; solid; shuffled | -0.07% | -0.79% | -0.87% | -1.07% | -1.72% | -1.35% | -6.29% | -0.84% | 0.69% |
| 1024 mixed strokes; 10% glyphs; gradient; shuffled | -0.69% | -1.71% | -0.18% | -0.42% | -1.66% | -0.11% | 0.81% | -16.70% | -1.40% |
| 512 rectangle fills; AA on | 1.56% | 0.39% | -1.29% | -1.34% | 4.03% | -0.58% | 0.78% | 9.11% | 9.92% |
| 64 rectangle strokes; AA off | 1.19% | 1.23% | 0.78% | 0.78% | 2.48% | 1.09% | -3.18% | 22.35% | 0.84% |

### Counter downsides and variation

- Rectangle fills, 512 draws with AA: branch misses increased 4.03% median (range 0.04% to 5.38%), positive in all five pairs. Median master/fix levels were 48,196.6 / 49,721.3 per measured-frame normalization.
- Rectangle strokes, 64 draws without AA: branch misses increased 2.48% median (range 1.88% to 5.62%), positive in all five pairs. Median master/fix levels were 2,493.7 / 2,568.4.
- Rectangle strokes also had a 22.35% median increase in L1I misses, but the paired range was −43.66% to 50.16% (IQR 9.11% to 36.53%). Rectangle-fill L1I misses increased 9.11% median with a −46.07% to 47.36% range. Those cache results are much less stable than the branch-miss increases.
- Moving glyph fills had a 10.86% median increase in dTLB misses, with a −2.03% to 99.73% range; retain the absolute counts and variation rather than treating that median as a stable effect.

These comparisons do not identify the responsible function or prove return-address-stack pressure. RAS occupancy and prefetch-miss counters were not measured. No attribution to the new transform-selection branch is assumed.

## Allocations and stage timings

Allocation calls, requested bytes, live bytes at start/end, and peak live requested bytes matched exactly in all 15 allocation pairs (five configurations × three rounds). Allocation builds use atomic instrumentation, so their timings are not timing evidence. The allocation phase used 16 measured frames to stay within the 32-bit requested-byte counter.

The separate five-pair [stage experiment](stages/summary.json) preserves every chronological draw/flush/frame sample and p50/p95/p99. Its frame clocks add overhead; the reported bulk comparisons above use plain bulk-mode runs. All 50 stage children were checked for sample lengths, draw+flush=frame, and matching summed stage totals.

## Artifacts and validation

- [Raw results archive](raw-results.tar.gz): 949,268 bytes; SHA-256 `cf44e62e8bab34b5704c23e5c6562c37eeda9ab17ab42eef874ac05bd80e163f` ([checksum file](raw-results.sha256)).
- The archive includes all per-child stdout/stderr, paired rows, perf CSVs and runtime percentages, stage arrays, pilots, job logs, two frozen source versions, optimized-helper disassembly, build/dependency/font hashes, and remote orchestration scripts. Font binaries and compiled executables are excluded; the pinned baseline supplies the hash-identified font.
- The source-v1/v2 distinction and validation-only label remain explicit in the archive and reports. A setup-only helper failed on an objdump address-format argument; its log is retained. The corrected helper ran before source-v2 compilation and measurement.
- Verified 948 measured child records / 474 exact-work pairs, including 60 validation-only children; source hashes matched a frozen snapshot for every dataset. All supported perf events reported 100% running. All child endpoints had performance/1.2 GHz. Eleven Python unit tests passed; the workload compiled against both pinned revisions locally and on the Pi.

```sh
tar -xzf raw-results.tar.gz
python3 ../../verify_stress.py out/pi3-389-stress-2026-10-09 --governor performance --frequency-khz 1200000 --require-perf-100
```

Run the verifier from the extracted archive directory, using the path to the repository’s `profiling/verify_stress.py` (the relative path above assumes extraction beside this README).

| Dataset | Configurations | Pairs/configuration | Interpretation | Reports |
| --- | ---: | ---: | --- | --- |
| sweep | 78 | 3 | Performance evidence | [table](sweep/summary.md), [full statistics](sweep/summary.json), [provenance](sweep/experiment.json) |
| confirm | 10 | 9 | Performance evidence | [table](confirm/summary.md), [full statistics](confirm/summary.json), [provenance](confirm/experiment.json) |
| perf-core | 5 | 5 | Performance evidence | [table](perf-core/summary.md), [full statistics](perf-core/summary.json), [provenance](perf-core/experiment.json) |
| perf-cache | 5 | 5 | Performance evidence | [table](perf-cache/summary.md), [full statistics](perf-cache/summary.json), [provenance](perf-cache/experiment.json) |
| stages | 5 | 5 | Performance evidence | [table](stages/summary.md), [full statistics](stages/summary.json), [provenance](stages/experiment.json) |
| allocations | 5 | 3 | Performance evidence | [table](allocations/summary.md), [full statistics](allocations/summary.json), [provenance](allocations/experiment.json) |
| long-validation | 3 | 10 | Collection validation only | [table](long-validation/summary.md), [full statistics](long-validation/summary.json), [provenance](long-validation/experiment.json) |
| long-clean | 3 | 10 | Performance evidence | [table](long-clean/summary.md), [full statistics](long-clean/summary.json), [provenance](long-clean/experiment.json) |

## Full exploratory sweep

All 78 configurations are shown here. These three-pair medians select candidates; they are not independently confirmed maxima.

| Configuration | Master ms/frame | PR ms/frame | Median paired change | Paired IQR |
| --- | ---: | ---: | ---: | ---: |
| 512 rectangle strokes; AA off | 2.313 | 2.561 | 10.73% | 4.85% to 10.88% |
| 64 rectangle fills; AA off | 0.482 | 0.521 | 8.53% | -0.04% to 8.66% |
| 1024 mixed strokes; 10% glyphs; gradient; grouped | 7.780 | 8.262 | 6.19% | -1.20% to 7.01% |
| 64 rectangle strokes; AA off | 0.266 | 0.286 | 5.80% | 4.11% to 6.63% |
| 512 rectangle fills; AA on | 4.329 | 4.535 | 4.78% | -2.23% to 5.75% |
| 1024 mixed fills; 50% glyphs; solid; grouped | 17.172 | 17.899 | 4.24% | 0.03% to 5.89% |
| 4096 rectangle strokes; AA on | 26.754 | 27.594 | 3.18% | -1.27% to 4.23% |
| 64 triangle strokes; AA on | 0.272 | 0.279 | 3.09% | -1.91% to 6.28% |
| 2048 glyph fills; 64 representations; shuffled; moving | 94.364 | 96.219 | 2.00% | 0.47% to 2.20% |
| 2048 glyph fills; 1 representations; grouped; moving | 51.663 | 52.080 | 1.94% | -0.93% to 2.23% |
| 4096 line strokes; AA off | 25.449 | 25.911 | 1.81% | 0.51% to 4.70% |
| 64 triangle fills; AA off | 0.421 | 0.420 | 1.63% | -2.77% to 3.72% |
| 2048 glyph fills; 1024 representations; shuffled; moving | 127.279 | 129.670 | 1.53% | 0.32% to 1.91% |
| 4096 triangle strokes; AA off | 26.279 | 26.428 | 0.68% | -0.04% to 0.72% |
| 512 rectangle strokes; AA on | 2.494 | 2.516 | 0.45% | -3.36% to 3.96% |
| 4096 triangle strokes; AA on | 25.320 | 26.414 | -0.41% | -1.08% to 3.43% |
| 1024 mixed strokes; 10% glyphs; solid; shuffled | 7.449 | 7.411 | -0.50% | -4.34% to 4.22% |
| 64 triangle strokes; AA off | 0.276 | 0.275 | -0.56% | -3.80% to 1.00% |
| 1024 mixed strokes; 10% glyphs; gradient; shuffled | 8.498 | 8.430 | -0.63% | -4.93% to 3.29% |
| 1024 mixed fills; 90% glyphs; solid; shuffled | 23.324 | 23.001 | -0.65% | -1.50% to 0.38% |
| 2048 glyph fills; 64 representations; shuffled; stationary | 93.435 | 92.755 | -0.73% | -0.85% to 0.37% |
| 2048 glyph fills; 1024 representations; grouped; stationary | 102.732 | 102.848 | -0.74% | -0.93% to -0.32% |
| 64 triangle fills; AA on | 0.488 | 0.459 | -0.84% | -3.40% to -0.78% |
| 2048 glyph fills; 16 representations; grouped; moving | 85.971 | 85.227 | -0.87% | -1.08% to -0.65% |
| 2048 glyph fills; 256 representations; shuffled; stationary | 100.083 | 99.138 | -0.87% | -1.20% to -0.22% |
| 4096 line strokes; AA on | 25.851 | 25.675 | -0.91% | -2.24% to -0.79% |
| 2048 glyph fills; 256 representations; grouped; stationary | 100.348 | 99.453 | -1.01% | -1.20% to -0.84% |
| 2048 glyph fills; 1024 representations; shuffled; stationary | 103.040 | 101.777 | -1.03% | -1.13% to -0.72% |
| 2048 glyph fills; 256 representations; grouped; moving | 105.650 | 104.833 | -1.08% | -1.23% to -0.93% |
| 1024 mixed fills; 10% glyphs; gradient; shuffled | 11.502 | 11.416 | -1.16% | -1.42% to -0.82% |
| 2048 glyph fills; 256 representations; shuffled; moving | 107.556 | 104.902 | -1.25% | -1.97% to -0.25% |
| 2048 glyph fills; 16 representations; shuffled; stationary | 85.774 | 84.632 | -1.30% | -1.38% to -0.20% |
| 2048 glyph fills; 16 representations; grouped; stationary | 86.391 | 85.084 | -1.43% | -3.16% to -1.38% |
| 2048 glyph fills; 64 representations; grouped; stationary | 96.064 | 94.621 | -1.45% | -2.51% to -1.39% |
| 2048 glyph fills; 4 representations; grouped; stationary | 68.675 | 67.587 | -1.58% | -2.42% to -1.49% |
| 1024 mixed strokes; 10% glyphs; solid; grouped | 7.853 | 7.760 | -1.61% | -3.91% to -0.36% |
| 64 rectangle strokes; AA on | 0.274 | 0.282 | -1.63% | -2.73% to 2.04% |
| 1024 mixed strokes; 50% glyphs; gradient; grouped | 13.965 | 13.681 | -1.68% | -5.90% to -1.67% |
| 64 rectangle fills; AA on | 0.512 | 0.521 | -1.71% | -4.06% to 0.57% |
| 2048 glyph fills; 64 representations; grouped; moving | 96.977 | 96.166 | -1.72% | -2.57% to 0.12% |
| 512 line strokes; AA off | 2.257 | 2.207 | -1.79% | -2.01% to -0.89% |
| 1024 mixed fills; 10% glyphs; solid; grouped | 11.474 | 11.266 | -1.81% | -1.97% to 2.15% |
| 4096 triangle fills; AA off | 36.278 | 35.592 | -1.89% | -2.43% to -1.49% |
| 1024 mixed fills; 10% glyphs; gradient; grouped | 11.796 | 11.613 | -1.89% | -2.07% to -1.59% |
| 1024 mixed fills; 90% glyphs; solid; grouped | 23.925 | 23.002 | -1.93% | -5.59% to -1.71% |
| 1024 mixed strokes; 90% glyphs; gradient; grouped | 18.245 | 17.968 | -1.95% | -4.22% to -0.29% |
| 1024 mixed strokes; 90% glyphs; solid; shuffled | 18.678 | 18.219 | -2.01% | -2.27% to -0.75% |
| 64 line strokes; AA on | 0.256 | 0.250 | -2.04% | -4.91% to 4.13% |
| 1024 mixed fills; 90% glyphs; gradient; shuffled | 23.777 | 23.500 | -2.06% | -2.38% to -1.31% |
| 2048 glyph fills; 4 representations; grouped; moving | 70.380 | 68.951 | -2.08% | -2.45% to -1.85% |
| 1024 mixed strokes; 50% glyphs; solid; shuffled | 13.739 | 13.510 | -2.30% | -2.39% to -1.98% |
| 1024 mixed fills; 50% glyphs; gradient; shuffled | 18.387 | 17.922 | -2.53% | -3.33% to -2.24% |
| 1024 mixed strokes; 50% glyphs; gradient; shuffled | 14.124 | 12.819 | -2.61% | -6.29% to -2.05% |
| 1024 mixed strokes; 90% glyphs; solid; grouped | 18.552 | 18.025 | -2.61% | -3.29% to -2.52% |
| 2048 glyph fills; 4 representations; shuffled; moving | 69.800 | 68.296 | -2.62% | -2.65% to -1.69% |
| 1024 mixed strokes; 50% glyphs; solid; grouped | 13.251 | 12.887 | -2.74% | -3.26% to -2.74% |
| 1024 mixed fills; 90% glyphs; gradient; grouped | 24.016 | 23.234 | -2.79% | -3.42% to -2.43% |
| 2048 glyph fills; 4 representations; shuffled; stationary | 69.851 | 67.681 | -2.91% | -4.16% to -2.79% |
| 1024 mixed strokes; 90% glyphs; gradient; shuffled | 19.143 | 18.609 | -3.05% | -3.74% to -1.62% |
| 4096 rectangle fills; AA off | 42.236 | 39.087 | -3.14% | -5.30% to -1.89% |
| 2048 glyph fills; 16 representations; shuffled; moving | 88.353 | 85.326 | -3.18% | -3.62% to -2.67% |
| 2048 glyph fills; 1024 representations; grouped; moving | 129.090 | 124.843 | -3.29% | -3.46% to -3.24% |
| 512 triangle strokes; AA on | 2.278 | 2.216 | -3.29% | -4.04% to -0.49% |
| 2048 glyph fills; 1 representations; grouped; stationary | 52.474 | 50.724 | -3.34% | -5.84% to -1.54% |
| 2048 glyph fills; 1 representations; shuffled; moving | 53.662 | 52.179 | -3.52% | -5.19% to -0.13% |
| 1024 mixed fills; 50% glyphs; solid; shuffled | 18.778 | 17.809 | -3.78% | -6.36% to -2.66% |
| 1024 mixed fills; 50% glyphs; gradient; grouped | 18.175 | 17.469 | -3.80% | -5.43% to -3.24% |
| 1024 mixed fills; 10% glyphs; solid; shuffled | 11.984 | 11.434 | -4.16% | -4.37% to -3.31% |
| 4096 rectangle strokes; AA off | 27.421 | 26.177 | -4.78% | -4.86% to -1.75% |
| 4096 rectangle fills; AA on | 45.251 | 42.675 | -6.45% | -7.35% to -3.83% |
| 512 rectangle fills; AA off | 4.168 | 3.801 | -6.58% | -7.80% to -4.82% |
| 64 line strokes; AA off | 0.264 | 0.247 | -6.86% | -7.46% to -0.16% |
| 512 line strokes; AA on | 2.264 | 2.123 | -6.91% | -7.98% to 1.54% |
| 512 triangle fills; AA on | 4.039 | 3.753 | -7.08% | -8.54% to -2.78% |
| 512 triangle strokes; AA off | 2.362 | 2.184 | -7.74% | -7.92% to -4.14% |
| 4096 triangle fills; AA on | 41.785 | 38.507 | -7.85% | -8.24% to -6.78% |
| 2048 glyph fills; 1 representations; shuffled; stationary | 52.413 | 48.261 | -7.92% | -8.65% to -4.85% |
| 512 triangle fills; AA off | 3.507 | 3.176 | -9.85% | -10.84% to -4.35% |
