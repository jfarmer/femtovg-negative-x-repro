#!/usr/bin/env python3
"""Portable adversarial sweeps for #389; uses the profiling runner's pairing and collectors."""
import argparse
import json
import math
import os
from pathlib import Path
import sys

import profile as common


def initial_plan():
    plan = []
    def add(case, **options):
        config = dict(case=case, draws=512, working_set=16, glyph_percent=50,
                      order="grouped", shape="triangle", paint="gradient", aa=True,
                      seed=389, motion="stationary")
        config.update(options)
        config["id"] = "-".join(str(config[k]).lower() for k in (
            "case", "draws", "working_set", "glyph_percent", "order", "shape", "paint", "aa", "seed", "motion"))
        plan.append(config)
    for case in ("tiny-path-fill", "tiny-path-stroke"):
        for shape in ("triangle", "rectangle", "line"):
            if case.endswith("fill") and shape == "line":
                continue
            for aa in (False, True):
                for draws in (64, 512, 4096):
                    add(case, shape=shape, aa=aa, draws=draws, paint="solid")
    for working_set in (1, 4, 16, 64, 256, 1024):
        for order in ("grouped", "shuffled"):
            for motion in ("stationary", "moving"):
                add("cache-glyph-fill", draws=2048, working_set=working_set, order=order, motion=motion)
    for case in ("mixed-fill", "mixed-stroke"):
        for ratio in (10, 50, 90):
            for order in ("grouped", "shuffled"):
                for paint in ("solid", "gradient"):
                    add(case, draws=1024, glyph_percent=ratio, order=order, paint=paint)
    return plan


def frames_for(ns_per_frame, seconds):
    # Stress motion repeats every 16 frames. Both variants cover identical full cycles.
    return min(8192, max(16, math.ceil(seconds * 1e9 / ns_per_frame / 16) * 16))


def argv_for(build, label, config, frames, warmup, measure, samples=None):
    argv = [build["binaries"][label]["path"], "--font", build["font"],
            "--case", config["case"], "--motion", config["motion"],
            "--frames", str(frames), "--warmup", str(warmup), "--measure", measure]
    for key in ("draws", "working_set", "glyph_percent", "order", "shape", "paint", "aa", "seed"):
        value = str(config[key]).lower() if isinstance(config[key], bool) else str(config[key])
        argv.extend(["--" + key.replace("_", "-"), value])
    if samples is not None:
        argv.extend(["--samples", str(samples)])
    return argv


def run(args):
    plan = json.loads(Path(args.plan).read_text())
    if not plan or len({c["id"] for c in plan}) != len(plan):
        raise ValueError("plan must have unique, nonempty scenario IDs")
    build = common.read_build(args.build)
    if args.collector == "perf" and (not sys.platform.startswith("linux") or build["instrumentation"] != "plain"):
        raise ValueError("perf requires Linux and plain instrumentation")
    out = common.new_result_dir(args.out)
    experiment = dict(schema_version=1, build=build, host=common.host(), plan=plan,
                      arguments=vars(args).copy(), notes=args.notes, schedule=[], frames_by_id={},
                      runner_sha256=common.digest(__file__), common_runner_sha256=common.digest(common.__file__))
    previous = json.loads((Path(args.frames_from) / "experiment.json").read_text())["frames_by_id"] if args.frames_from else {}
    if args.collector == "perf":
        experiment["perf_version"] = common.capture(["perf", "--version"])
        experiment["collector_environment"] = {"LC_ALL": "C"}
        with open(out / "perf-list.txt", "w") as stream:
            common.command(["perf", "list"], stdout=stream)
    for config in plan:
        if config["id"] in previous:
            frames = previous[config["id"]]
        else:
            print("calibrating " + config["id"], flush=True)
            argv = argv_for(build, "master", config, 16, args.warmup, "bulk")
            name = "pilot-" + config["id"]
            process = common.execute(argv, out / (name + ".stdout.json"), out / (name + ".stderr.txt"), args.cpu)
            result = json.loads((out / (name + ".stdout.json")).read_text())
            frames = frames_for(result["elapsed_ns"] / result["frames"], args.target_seconds)
            with open(out / "pilots.jsonl", "a") as stream:
                stream.write(json.dumps(dict(config=config, command=argv, result=result, process=process)) + "\n")
        experiment["frames_by_id"][config["id"]] = frames
    rows = []
    for repeat, config, labels in common.paired_schedule(plan, args.repeats):
        frames = experiment["frames_by_id"][config["id"]]
        for label in labels:
            name = f"{repeat:03}-{config['id']}-{label}"
            argv = argv_for(build, label, config, frames, args.warmup, args.measure,
                            out / (name + ".frames.json") if args.measure == "stages" else None)
            counter = out / (name + ".perf.csv")
            if args.collector == "perf":
                argv = ["perf", "stat", "--no-big-num", "-x", ";", "-o", str(counter), "-e", args.events, "--"] + argv
            job = dict(name=name, command=argv, label=label, repeat=repeat, config=config)
            experiment["schedule"].append(job)
            common.write_json(out / "experiment.json", experiment)
            process = common.execute(argv, out / (name + ".stdout.json"), out / (name + ".stderr.txt"), args.cpu,
                                     dict(os.environ, LC_ALL="C") if args.collector == "perf" else None)
            result = json.loads((out / (name + ".stdout.json")).read_text())
            result["workload_case"] = result["case"]
            result["case"] = config["id"]
            row = dict(**job, result=result, process=process,
                       perf=common.perf_events(counter) if args.collector == "perf" else None)
            rows.append(row)
            with open(out / "runs.jsonl", "a") as stream:
                stream.write(json.dumps(row) + "\n")
            print(f"{name}: {result['elapsed_ns'] / frames / 1000:.2f} us/frame ({frames} frames)", flush=True)
    common.verify_build(build)
    common.report_rows(rows, out / "summary.json", out / "summary.md", experiment)
    print(out / "summary.md", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write-plan", help="write the full initial sweep and exit")
    parser.add_argument("--plan")
    parser.add_argument("--build", default=str(common.ROOT / "target/profiling/plain/build.json"))
    parser.add_argument("--out")
    parser.add_argument("--frames-from", help="reuse calibrated frame counts from a previous experiment")
    parser.add_argument("--target-seconds", type=float, default=0.6)
    parser.add_argument("--warmup", type=int, default=16)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--measure", choices=("bulk", "stages"), default="bulk")
    parser.add_argument("--collector", choices=("none", "perf"), default="none")
    parser.add_argument("--events", default="cycles,instructions,branches,branch-misses")
    parser.add_argument("--cpu", type=int)
    parser.add_argument("--notes", default="")
    args = parser.parse_args()
    if args.write_plan:
        common.write_json(args.write_plan, initial_plan())
        return
    if not args.plan or not args.out:
        parser.error("--plan and --out are required")
    if not math.isfinite(args.target_seconds) or args.target_seconds <= 0 or args.warmup < 0 or args.repeats < 1:
        parser.error("target-seconds/repeats must be positive and warmup nonnegative")
    try:
        run(args)
    except (ValueError, OSError, RuntimeError) as error:
        parser.exit(1, f"stress.py: {error}\n")


if __name__ == "__main__":
    main()
