"""Compare the repository's existing scrolling table and label scenes on Metal."""
import json
import os
from pathlib import Path
import re
import shutil
from statistics import median
import subprocess

from run import HERE, OUT


def build(side):
    # Reuse the original harness, manifest and lock from this repro checkout.
    repo = HERE.parent
    root = OUT / "textframe" / side
    project = root / "crates/textframe-master"
    project.mkdir(parents=True, exist_ok=True)
    shutil.copytree(repo / "crates/textframe-master", project, dirs_exist_ok=True)
    subprocess.run(["patch", "-p1", "-i", str(HERE / "textframe-lock.patch")], cwd=root, check=True)
    (root / "femtovg").mkdir(exist_ok=True)
    for link, source in [(root / "src", repo / "src"), (root / "femtovg/textframe-master", OUT / side / "femtovg")]:
        if not source.is_dir():
            raise SystemExit("Run settings-labels/run.py build metal first")
        if not link.exists():
            link.symlink_to(source, target_is_directory=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", OUT / "target")).resolve()
    subprocess.run(["cargo", "build", "--locked", "--release", "--no-default-features", "--features", "swash"],
                   cwd=project, env=dict(os.environ, CARGO_TARGET_DIR=str(target)), check=True)
    binary = root / "bench"
    shutil.copy(target / "release/textframe-master", binary)
    return binary


def sample(binary, scene, frames, pixels=False):
    env = dict(os.environ, FONT=str(HERE.parent / "fonts/Roboto-Regular.ttf"))
    if pixels:
        env["PIXEL_HASH"] = "1"
    result = subprocess.check_output([binary, "glyph_run", str(frames), "wgpu", scene], text=True, env=env)
    if pixels:
        return re.search(r"hash=(\w+)", result)[1]
    return float(re.search(r"median_us=([\d.]+)", result)[1])


if __name__ == "__main__":
    binaries = {"master": build("base"), "fixed": build("both")}
    results = {}
    for scene in ["table", "labels"]:
        if len({sample(b, scene, 3, pixels=True) for b in binaries.values()}) != 1:
            raise SystemExit(f"{scene}: pixel check failed")
        rows = []
        for i in range(16):
            order = list(binaries) if i % 2 == 0 else list(reversed(binaries))
            rows.append({side: sample(binaries[side], scene, 201) for side in order})
        results[scene] = rows
        before = median(row["master"] for row in rows) / 1000
        after = median(row["fixed"] for row in rows) / 1000
        change = median(row["fixed"] / row["master"] - 1 for row in rows)
        print(f"{scene}: {before:.3f} -> {after:.3f} ms/frame ({change:+.1%})", flush=True)
    (OUT / "textframe.json").write_text(json.dumps(results) + "\n")
