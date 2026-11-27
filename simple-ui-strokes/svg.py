"""Run from a scratch directory containing builds/{master,after}/svg-harness."""
import json
import subprocess
from pathlib import Path

scenes = ["inspector", "borders", "charts"]
source = Path(__file__).resolve().parent / "scenes"
options = ["--text", "glyphs", "--frame", "1920x1080", "--box", "960@480,220"]


def command(build, action):
    return [f"builds/{build}/svg-harness", action, *options]


for scene in scenes:
    for build in ("master", "after"):
        subprocess.run(command(build, "render") + [source / f"{scene}.svg", f"{scene}-{build}.png"], check=True)
    if Path(f"{scene}-master.png").read_bytes() != Path(f"{scene}-after.png").read_bytes():
        raise SystemExit(f"render mismatch: {scene}")

with open("svg.jsonl", "x") as output:
    for pair in range(20):
        offset = pair % len(scenes)
        for scene in scenes[offset:] + scenes[:offset]:
            row = {"pair": pair, "scene": scene}
            for build in (("master", "after") if pair % 2 == 0 else ("after", "master")):
                timing = subprocess.check_output(
                    command(build, "time") + ["--frames", "201", source / f"{scene}.svg"], text=True
                )
                row[build] = json.loads(timing.splitlines()[-1])
            print(json.dumps(row), file=output, flush=True)
