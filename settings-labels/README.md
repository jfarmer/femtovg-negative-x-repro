# Color glyph checks on positioned text

The **settings panel** draws 60 labels in a 900 × 900 offscreen target using `fill_glyph_run()`, the API Slint calls for positioned text. Six strings repeat across a 20 × 3 grid. The glyph IDs are looked up before timing; positions advance nine pixels per character. This is a synthetic renderer workload. The [scene](src/lib.rs) is shared by the Metal, macOS OpenGL and CPU-only Void runners.

The **scrolling orders table** and **200 labels** come from this repository’s existing [textframe benchmark](../src/textframe.rs). The table advances 7.25 px per frame through 5,000 rows and eight columns. Each frame draws the visible rows clipped to the area below the header (about 390 cells). The labels scene draws 200 separately positioned strings of 10–40 characters. Both use the bundled Roboto Regular font, its glyph advances, and a warm atlas.

## Reproduce

The settings runner requires Python 3, Git, Cargo and a Rust toolchain supported by femtovg. The initial build fetches the pinned library source and Cargo dependencies. Metal and OpenGL use macOS; the Void backend runs without a display. From the repository root:

```sh
python3 settings-labels/run.py build metal
python3 settings-labels/run.py measure metal
python3 settings-labels/run.py build opengl
python3 settings-labels/run.py measure opengl
```

On the Pi 3, pin each timed process to one core with Linux `taskset`:

```sh
CARGO_BUILD_JOBS=2 python3 settings-labels/run.py build void
python3 settings-labels/run.py measure void --core 2
```

For the two existing workloads, run the following on a Mac after `build metal`. This builds the existing textframe source against the baseline and both commits, checks a frame hash in each scene, and runs 16 alternating pairs:

```sh
python3 settings-labels/textframe.py
```

Builds, frame captures and fresh results go under `out/settings-labels/`. To reproduce the settings table from the **archived samples** without compiling anything:

```sh
python3 settings-labels/run.py archived metal
python3 settings-labels/run.py archived opengl
python3 settings-labels/run.py archived void
```

## Comparisons

| Variant | femtovg source |
| :--- | :--- |
| Baseline | `786d75778291fc579be9a013466e10ddd80a7884` |
| Bitmap only | `4b7a9dc3baa4d9b14d97091c0c8846b09b639b85` |
| COLR only | Baseline plus [the standalone change](colr-only.patch), equivalent to `b4083458858dd7b8c787a09a2e4e20cb65bcddbf`. The runner verifies the resulting font source blob. |
| Both | `72b6ec573e92d1c8e5d346bd91033c1bcce8aa6d` |

All settings builds disable default features. `swash` enables Swash alone; `mixed` also enables `textlayout`. The bitmap shortcut compiles only in the former configuration. The runner uses the Roboto Flex font from the pinned femtovg checkout. Set `FEMTOVG_URL` to use a local clone containing the commits.

Each settings run excludes the first frame and takes the median over the next 200. Variant order rotates and reverses across 16 rounds. The time reported for a build is the median of its run medians. The percentage is the median of the within-round changes from baseline. The M4 Max CPU timer ends at canvas flush, before `glFinish` or Metal queue submission. The second GPU timing includes completion. The ARMv7 Pi 3 measures CPU preparation and flush on the Void renderer. Pixel comparisons are within the same GPU backend and feature configuration.

[Metal](results/metal.json) and [OpenGL](results/opengl.json) samples contain pairs of nanoseconds (CPU encoding, completed frame). The [Pi samples](results/void.txt) record round, features, variant and CPU nanoseconds. These samples were collected before factoring the common scene into the published runner.

## Existing textframe results

M4 Max, wgpu/Metal, 1920 × 1080, Swash only, using baseline and both commits. Times **include GPU completion**. The original harness warms up for two frames, then measures 201 frames per run. Both scenes improved in all 16 pairs, and baseline and patched frame hashes matched in both scenes.

| Scene | Baseline (ms/frame) | Both | Paired change | Pair range |
| :--- | ---: | ---: | ---: | ---: |
| Scrolling orders table | 0.899 | 0.771 | **−14.7%** | −18.0% to −1.7% |
| 200 labels | 0.726 | 0.618 | **−14.8%** | −24.4% to −11.0% |

The [original samples](results/textframe.json) store the harness’s per-run median in microseconds.
