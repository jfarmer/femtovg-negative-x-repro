# Simple UI strokes

Compares femtovg master `786d75778291fc579be9a013466e10ddd80a7884` with [the simple strokes patch](https://github.com/jfarmer/femtovg/commit/095d6c55b80a7461f81589697918329d84c8820c). Measurements were taken on an M4 Max using Metal.

## SVG workloads

The [six charts](scenes/charts.svg) and [property inspector](scenes/inspector.svg) are generated UI scenes. The [400 separate rectangular borders](scenes/borders.svg) isolate the rectangle fast path. Each workload was rendered with femtovg’s svg-harness, with glyph text and a 1920 × 1080 target. [Paired samples](results/svg.jsonl) include CPU encoding and time through GPU completion. Frame one is excluded. The reported change is the median within-pair percentage over 20 alternating pairs, with 200 measured frames per run.

To reproduce on macOS, set `REPRO` to this repository and `FEMTOVG` to a clone containing both femtovg commits. In a fresh scratch directory:

```sh
export SVG_HARNESS_OUT="$PWD"
python3 "$FEMTOVG/tools/svg-harness/scripts/build.py" master 786d75778291fc579be9a013466e10ddd80a7884
python3 "$FEMTOVG/tools/svg-harness/scripts/build.py" after 095d6c55b80a7461f81589697918329d84c8820c
python3 "$REPRO/simple-ui-strokes/svg.py"
```

## Slint spreadsheet

The [7GUIs Cells UI](https://github.com/slint-ui/slint/blob/02bb0be28ad031bb84240ae752561f508e1e18e4/examples/7guis/cells.slint#L39-L49) gives each cell a one-pixel border. The benchmark uses the example’s [stock model](https://github.com/slint-ui/slint/blob/02bb0be28ad031bb84240ae752561f508e1e18e4/examples/7guis/cells.rs) and a [scrolling and timing driver](cells.patch). It scrolls down and back at 650 logical pixels/s in a 1440 × 850 window at 2× display scale. The timer spans Slint’s BeforeRendering and AfterRendering callbacks, ending after submission and before presentation. [Paired results](results/cells.csv) give the median renderer time and callback rate for each run.

To reproduce, set `SLINT` to a checkout of the linked Slint revision, `BASE` and `PATCHED` to the respective femtovg checkouts, and `REPRO` as above. In a separate empty directory:

```sh
mkdir src
cp "$SLINT/examples/7guis/cells.rs" src/main.rs
cp "$SLINT/examples/7guis/cells.slint" src/
patch -p1 < "$REPRO/simple-ui-strokes/cells.patch"
cp "$REPRO/simple-ui-strokes/cells.lock" Cargo.lock
cat > Cargo.toml <<TOML
[package]
name = "stroke-cells"
version = "0.1.0"
edition = "2024"

[dependencies]
slint = { path = "$SLINT/api/rs/slint", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-femtovg-wgpu"] }
TOML
for side in master patched; do
    if [ "$side" = master ]; then lib="$BASE"; else lib="$PATCHED"; fi
    cargo build --locked --release --target-dir "target-$side" --config "patch.crates-io.femtovg.path='$lib'"
    cp "target-$side/release/stroke-cells" "$side"
done
```

Run `SLINT_BACKEND=winit-femtovg-wgpu ./master` and `./patched` with the same backend, alternating order across 14 pairs. Each process prints elapsed seconds and renderer milliseconds for each callback. Take the median milliseconds in the 2.5–7.5 second interval for each run, then the median percentage change across pairs. Both builds sustained about 120 render callbacks/s.
