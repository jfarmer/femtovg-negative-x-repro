import argparse
import csv
import math
import statistics
import subprocess as sp
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

parser = argparse.ArgumentParser(
    description="Encode both zoom comparisons as five-second H.264 videos."
)
parser.add_argument("--work", type=Path, default=Path("out/contour-motion"))
parser.add_argument("--output", type=Path, default=Path("images/contour-motion"))
parser.add_argument(
    "--measured-on",
    help="Use fresh timings and paired images. Supply a hardware/backend label.",
)
args = parser.parse_args()
out = args.output
out.mkdir(exist_ok=True, parents=True)
tags = ["baseline", "scan-48"]
font = "/System/Library/Fonts/Supplemental/Arial.ttf"
bold = "/System/Library/Fonts/Supplemental/Arial Bold.ttf"


def f(n, b=False):
    return ImageFont.truetype(bold if b else font, n)


for name, title, sub in [
    ("atlas", "Zooming a six-map atlas", "6 copies of the Natural Earth land path"),
    ("labels", "Zooming a QR label sheet", "12 copies of the same vector contact code"),
]:
    p = args.work / name
    N = 300
    durations = {}
    paths = {}
    for tag in tags:
        csv_path = (
            p / (tag + ".csv")
            if args.measured_on
            else Path(__file__).resolve().parent / "timings" / f"{name}-{tag}.csv"
        )
        with csv_path.open() as trace:
            durations[tag] = [
                float(r["wall_ms"])
                for r in csv.DictReader(trace)
                if r.get("pass", "1") == "1"
            ]
        if len(durations[tag]) != N:
            raise ValueError(f"Expected {N} timings in {csv_path}")
        source = tag if args.measured_on else "baseline"
        paths[tag] = [p / f"{source}.{j:02}.png" for j in range(N)]
    if args.measured_on:
        for a, b in zip(*[paths[t] for t in tags]):
            with Image.open(a) as left, Image.open(b) as right:
                if (
                    left.size != right.size
                    or left.convert("RGBA").tobytes() != right.convert("RGBA").tobytes()
                ):
                    raise ValueError(f"Different renderer output at {a.name}")
    screens = {}
    rates = {}
    for tag in tags:
        shown = 0
        next_ready = 0
        pending = 0
        frames = []
        changed = 0
        for tick in range(600):
            if tick >= next_ready:
                if tick:
                    changed += 1
                shown = pending
                pending = (tick * 2) % N
                next_ready = tick + max(
                    1, math.ceil(durations[tag][pending] * 60 / 1000)
                )
            frames.append(shown)
        screens[tag] = frames
        rates[tag] = changed / 10
    W, H = 1488, 670
    base = Image.new("RGB", (W, H), "#f7f6f1")
    d = ImageDraw.Draw(base)
    d.text((16, 10), title, font=f(36, True), fill="#182c30")
    d.text((16, 55), sub, font=f(20), fill="#495a5b")
    for idx, tag in enumerate(tags):
        x = 16 + 736 * idx
        d.text((x, 94), ["Master", "Patched"][idx], font=f(25, True), fill="#162c30")
        ms = statistics.median(durations[tag])
        stat = f"{ms:.1f} ms / draw    ·    {rates[tag]:.0f} updates/s"
        d.text((x + 720, 98), stat, font=f(21), fill="#304e52", anchor="rt")
    d.text(
        (16, 640),
        f"{args.measured_on or 'M4 Max  /  Metal'}     ·     60 Hz replay using measured draw times     ·     Same zoom gesture on both sides",
        font=f(18),
        fill="#405658",
    )
    proc = sp.Popen(
        [
            "ffmpeg",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            f"{W}x{H}",
            "-framerate",
            "60",
            "-i",
            "-",
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "17",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            str(p / "full.mp4"),
        ],
        stdin=sp.PIPE,
    )
    imgs = {}
    preview = []
    for j in range(600):
        frame = base.copy()
        for idx, tag in enumerate(tags):
            i = screens[tag][j]
            if i not in imgs:
                with Image.open(paths[tag][i]) as im:
                    imgs[i] = im.convert("RGB")
            frame.paste(imgs[i], (16 + 736 * idx, 132))
        # One complete zoom cycle at the same 60 Hz as the full-size clip.
        if j < 150:
            preview.append(frame.resize((960, 432), Image.Resampling.LANCZOS))
        proc.stdin.write(frame.tobytes())
    proc.stdin.close()
    assert proc.wait() == 0
    print(
        name,
        [
            (tag, round(statistics.median(durations[tag]), 2), rates[tag])
            for tag in tags
        ],
        flush=True,
    )
    sp.run(
        [
            "ffmpeg",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(p / "full.mp4"),
            "-t",
            "5",
            "-c:v",
            "libx264",
            "-crf",
            "19",
            "-maxrate",
            "12M",
            "-bufsize",
            "8M",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            str(out / (name + ".mp4")),
        ],
        check=True,
    )

    # Quantize refresh deadlines, not individual durations, to whole milliseconds.
    ticks = [round(i * 1000 / 60) for i in range(len(preview) + 1)]
    preview[0].save(
        out / (name + ".webp"),
        save_all=True,
        append_images=preview[1:],
        duration=[end - start for start, end in zip(ticks, ticks[1:])],
        loop=0,
        quality=72,
        method=4,
    )
