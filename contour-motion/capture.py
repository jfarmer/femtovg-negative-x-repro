import argparse
import math
import subprocess as sp
import xml.etree.ElementTree as ET
from pathlib import Path

parser = argparse.ArgumentParser(
    description="Render the zoom gestures and optionally measure both builds."
)
parser.add_argument("baseline", type=Path, help="Master svg-harness binary")
parser.add_argument(
    "--patched", type=Path, help="Capture a new comparison with this second binary"
)
parser.add_argument("--work", type=Path, default=Path("out/contour-motion"))
args = parser.parse_args()
work = args.work
root = Path(__file__).resolve().parent / "fixtures"
zooms = ",".join(
    f"{1 + 0.24 * (1 - math.cos(2 * math.pi * i / 300)):.6f}" for i in range(300)
)
for name, fixture in [("atlas", "land"), ("labels", "contact")]:
    p = work / name
    p.mkdir(parents=True, exist_ok=True)
    path = ET.parse(root / (fixture + ".svg")).getroot().find("{*}path").attrib["d"]
    if name == "atlas":
        bg = "#eff4f4"
        parts = []
        for i in range(6):
            x = 10 + 262 * (i % 2)
            y = 13 + 119 * (i // 2)
            parts.append(
                f'<rect x="{x}" y="{y}" width="248" height="110" rx="4" fill="#d4e7ea"/><g transform="translate({x + 15} {y + 1}) scale(.605)"><path fill="{["#467367", "#447587", "#707b4a"][i % 3]}" fill-rule="evenodd" d="{path}"/></g>'
            )
    else:
        bg = "#e8e6e1"
        parts = []
        for i in range(12):
            x = 10 + 131 * (i % 4)
            y = 9 + 119 * (i // 4)
            parts.append(
                f'<rect x="{x}" y="{y}" width="119" height="109" rx="4" fill="#fff"/><rect x="{x + 8}" y="{y + 8}" width="3" height="92" fill="{["#d75a31", "#347a6f", "#6084af"][i % 3]}"/><g transform="translate({x + 18} {y + 8}) scale(1.17)"><path fill="#262829" d="{path}"/></g>'
            )
    f = p / "scene.svg"
    f.write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="536" height="372" viewBox="0 0 536 372"><rect width="536" height="372" fill="{bg}"/>'
        + "".join(parts)
        + "</svg>"
    )
    (p / "list.txt").write_text(str(f.resolve()) + "\n")
    for tag in ["baseline", "scan-48"] if name == "atlas" else ["scan-48", "baseline"]:
        if tag == "scan-48" and args.patched is None:
            continue
        binary = (args.baseline if tag == "baseline" else args.patched).resolve()
        opts = [
            "--frame",
            "720x500",
            "--box",
            "720@0,0",
            "--pivot",
            "360,250",
            "--zooms",
            zooms,
        ]
        if args.patched:
            sp.run(
                [
                    binary,
                    "soak",
                    *opts,
                    "--passes",
                    "2",
                    p / "list.txt",
                    p / (tag + ".csv"),
                ],
                check=True,
            )
        sp.run([binary, "ladder", *opts, f, p / tag], check=True)
