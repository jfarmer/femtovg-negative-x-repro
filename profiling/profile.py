#!/usr/bin/env python3
"""Build and compare #389 workloads. Standard library only; no shell command strings."""
import argparse
import datetime as dt
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
BASE = "6dd55434177690c845fc5e6f8e9d5a2fe6f1b277"
FIXED = "38d649c699b339f040a2e8f8a13ed28e1644baf7"
CASES = ("gradient-fill", "gradient-stroke", "glyph-fill", "glyph-stroke", "dashed-stroke", "image-fill",
         "shadow-fill", "transformed-fill", "atlas-fill", "path-fill", "path-stroke")


def command(argv, **kwargs):
    return subprocess.run([str(a) for a in argv], check=True, **kwargs)


def capture(argv):
    return command(argv, stdout=subprocess.PIPE, text=True).stdout.strip()


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def linux_snapshot(sysfs=Path("/sys"), cpu=None):
    """Optional sysfs readings; no writes, platform commands, or continuous sampler."""
    def read(path, number=False):
        try:
            value = path.read_text().strip()
            return int(value) if number else value
        except (OSError, ValueError):
            return None
    temperatures = []
    for zone in sorted((sysfs / "class/thermal").glob("thermal_zone*")):
        temperatures.append({"zone": zone.name, "type": read(zone / "type"),
                             "temperature_millicelsius": read(zone / "temp", True)})
    cpu_root = sysfs / "devices/system/cpu"
    paths = [cpu_root / f"cpu{cpu}"] if cpu is not None else sorted(cpu_root.glob("cpu[0-9]*"))
    frequencies = {}
    for path in paths:
        policy = path / "cpufreq"
        if not policy.exists():
            continue
        frequencies[path.name] = {"governor": read(policy / "scaling_governor"),
                                 "driver": read(policy / "scaling_driver")}
        for key in ("scaling_cur_freq", "scaling_min_freq", "scaling_max_freq",
                    "cpuinfo_cur_freq", "cpuinfo_min_freq", "cpuinfo_max_freq"):
            frequencies[path.name][key + "_khz"] = read(policy / key, True)
    return {"utc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "thermal_zones": temperatures, "cpu_frequency": frequencies}


def host():
    data = {"platform": platform.platform(), "machine": platform.machine(),
            "processor": platform.processor(), "python": sys.version,
            "cpu_count": os.cpu_count(), "utc": dt.datetime.now(dt.timezone.utc).isoformat()}
    if sys.platform == "darwin":
        for key, argv in (("cpu_brand", ["sysctl", "-n", "machdep.cpu.brand_string"]),
                          ("xcode", ["xcodebuild", "-version"])):
            try:
                data[key] = capture(argv)
            except (OSError, subprocess.CalledProcessError) as error:
                data[key] = None
                data[f"{key}_error"] = str(error)
    elif sys.platform.startswith("linux"):
        cpuinfo = Path("/proc/cpuinfo")
        if cpuinfo.exists():
            data["cpuinfo"] = cpuinfo.read_text()
        if hasattr(os, "sched_getaffinity"):
            data["cpu_affinity"] = sorted(os.sched_getaffinity(0))
        data["linux_sysfs"] = linux_snapshot()
    return data


def checkout(path, source, revision):
    if not (path / ".git").exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        command(["git", "init", "--quiet", path])
    status = capture(["git", "-C", path, "status", "--porcelain", "--untracked-files=all"])
    if status:
        raise ValueError(f"refusing to change dirty checkout {path}:\n{status}")
    result = subprocess.run(["git", "-C", str(path), "rev-parse", "--verify", "HEAD"],
                            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    if result.returncode or result.stdout.strip() != revision:
        source_path = Path(source).expanduser()
        source = str(source_path.resolve()) if source_path.exists() else source
        command(["git", "-C", path, "fetch", "--quiet", "--depth", "1", source, revision])
        command(["git", "-C", path, "checkout", "--quiet", "--detach", "FETCH_HEAD"])
    return capture(["git", "-C", path, "rev-parse", "HEAD"])


def prepare(args):
    out = Path(args.out).resolve() if args.out else ROOT / "target/profiling" / args.instrumentation
    cargo_target = Path(args.cargo_target_dir).resolve() if args.cargo_target_dir else out / "cargo"
    out.mkdir(parents=True, exist_ok=True)
    sources = {}
    for label, source, rev in (("master", args.base_source, args.base_rev),
                               ("fixed", args.fixed_source, args.fixed_rev)):
        path = ROOT / "femtovg" / f"gradient-{label}"
        sources[label] = {"path": str(path), "revision": checkout(path, source, rev)}
    # Read the raw commit, including a parent hidden by a shallow checkout.
    commit = capture(["git", "-C", sources["fixed"]["path"], "cat-file", "-p", "HEAD"])
    parents = [line.split()[1] for line in commit.splitlines() if line.startswith("parent ")]
    if parents != [sources["master"]["revision"]]:
        raise ValueError("the fixed revision must be a single commit on the baseline; use an isolated pair")
    font = Path(args.font).resolve() if args.font else Path(sources["master"]["path"]) / "examples/assets/RobotoFlex-VariableFont.ttf"
    # Own the font used by this build; later repro runs may change the checkout.
    shutil.copyfile(font, out / "font.ttf")
    metadata = {"schema_version": 1, "host": host(), "rustc": capture(["rustc", "-Vv"]),
                "cargo": capture(["cargo", "-V"]), "instrumentation": args.instrumentation,
                "environment": {k: v for k, v in os.environ.items() if k in
                                ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET")
                                or k.startswith("CARGO_PROFILE_RELEASE_")
                                or (k.startswith("CARGO_TARGET_") and k.endswith("_RUSTFLAGS"))},
                "sources": sources, "cargo_target_dir": str(cargo_target), "font": str(out / "font.ttf"),
                "font_sha256": digest(out / "font.ttf"), "binaries": {},
                "harness_sha256": {str(p.relative_to(ROOT)): digest(p) for p in
                                   (ROOT / "profiling/workload.rs", ROOT / "profiling/build.rs", ROOT / "profiling/signposts.c", Path(__file__))}}
    for label in ("master", "fixed"):
        manifest = ROOT / f"crates/profile-{label}/Cargo.toml"
        argv = ["cargo", "build", "--release", "--locked", "--manifest-path", manifest,
                "--target-dir", cargo_target]
        if args.offline:
            argv.append("--offline")
        if args.instrumentation != "plain":
            argv.extend(["--features", args.instrumentation])
        # Cargo reports the executable path, including target-triple directories.
        argv.extend(["--message-format", "json"])
        output = capture(argv)
        executable = None
        for line in output.splitlines():
            event = json.loads(line)
            if event.get("reason") == "compiler-artifact" and event.get("executable"):
                executable = Path(event["executable"])
        if executable is None:
            raise ValueError("cargo did not report the workload executable")
        target = out / (f"profile-{label}" + executable.suffix)
        # A fresh inode also avoids stale macOS code-signature cache entries
        # when a previous version of this executable has already been launched.
        temporary = target.with_name(target.name + ".new")
        shutil.copy2(executable, temporary)
        temporary.replace(target)
        dsym = Path(str(executable) + ".dSYM")
        if dsym.exists():
            shutil.copytree(dsym, Path(str(target) + ".dSYM"), dirs_exist_ok=True)
        metadata["binaries"][label] = {"path": str(target), "sha256": digest(target),
                                          "lock_sha256": digest(manifest.with_name("Cargo.lock")),
                                          "manifest_sha256": digest(manifest)}
    # Check dependency resolution matches, ignoring the two root package names.
    # Ask Cargo instead of depending on Python 3.11's tomllib.
    dependencies = []
    for label in ("master", "fixed"):
        argv = ["cargo", "metadata", "--locked", "--format-version", "1",
                "--manifest-path", ROOT / f"crates/profile-{label}/Cargo.toml"]
        if args.offline:
            argv.append("--offline")
        data = json.loads(capture(argv))
        dependencies.append(sorted((p["name"], p["version"], p["source"]) for p in data["packages"]
                                   if not p["name"].startswith("repro-profile-")))
    if dependencies[0] != dependencies[1]:
        raise ValueError("baseline and fixed dependencies differ")
    metadata["dependencies"] = dependencies[0]
    write_json(out / "build.json", metadata)
    print(out / "build.json")


def read_build(path):
    build = json.loads(Path(path).read_text())
    verify_build(build)
    return build


def verify_build(build):
    for artifact in [build["font"]] + [b["path"] for b in build["binaries"].values()]:
        expected = build["font_sha256"] if artifact == build["font"] else next(
            b["sha256"] for b in build["binaries"].values() if b["path"] == artifact)
        if digest(artifact) != expected:
            raise ValueError(f"artifact changed since prepare: {artifact}")


def workload(build, label, case, motion, text, lines, args, seconds=0.0):
    return [build["binaries"][label]["path"], "--font", build["font"], "--case", case,
            "--motion", motion, "--text", text, "--lines", str(lines), "--frames", str(args.frames),
            "--warmup", str(args.warmup), "--measure", args.measure, "--qos", args.qos,
            "--seconds", str(seconds)]


def execute(argv, stdout_path, stderr_path, cpu=None, env=None):
    # wait4 returns resource usage for THIS process, not the accumulated maximum
    # of earlier children. Files avoid pipe deadlocks, even with verbose profilers.
    affinity = None
    if cpu is not None:
        if not hasattr(os, "sched_setaffinity"):
            raise ValueError("--cpu requires Linux sched_setaffinity")
        affinity = lambda: os.sched_setaffinity(0, {cpu})
    before = linux_snapshot(cpu=cpu) if sys.platform.startswith("linux") else None
    start = time.perf_counter_ns()
    with open(stdout_path, "w") as stdout, open(stderr_path, "w") as stderr:
        proc = subprocess.Popen(argv, stdout=stdout, stderr=stderr, preexec_fn=affinity, env=env)
        if hasattr(os, "wait4"):
            _, status, usage = os.wait4(proc.pid, 0)
            proc.returncode = os.waitstatus_to_exitcode(status)
            # Darwin uses bytes; Linux and BSD report KiB. Unknown hosts keep native units.
            rss = usage.ru_maxrss if sys.platform == "darwin" else (
                usage.ru_maxrss * 1024 if sys.platform.startswith(("linux", "freebsd")) else None)
            resources = {"user_cpu_seconds": usage.ru_utime, "system_cpu_seconds": usage.ru_stime,
                         "max_rss_bytes": rss, "minor_faults": usage.ru_minflt,
                         "major_faults": usage.ru_majflt, "voluntary_context_switches": usage.ru_nvcsw,
                         "involuntary_context_switches": usage.ru_nivcsw}
        else:
            proc.wait()
            resources = None
    wall_ns = time.perf_counter_ns() - start
    after = linux_snapshot(cpu=cpu) if sys.platform.startswith("linux") else None
    if proc.returncode:
        raise RuntimeError(f"command exited {proc.returncode}: {argv}\n{Path(stderr_path).read_text()}")
    return {"wall_ns": wall_ns, "resources": resources,
            "environment_before": before, "environment_after": after}


def new_result_dir(path):
    path = Path(path).resolve()
    # No stale rows or overwritten traces from a previous experiment.
    path.mkdir(parents=True, exist_ok=False)
    return path


def selection(value, choices):
    values = value.split(",")
    if not values or any(v not in choices for v in values) or len(set(values)) != len(values):
        raise argparse.ArgumentTypeError(f"choose unique comma-separated values from {','.join(choices)}")
    return values


def perf_events(path):
    # perf's machine-readable delimiter format; unsupported counts stay null.
    events = []
    for line in Path(path).read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        fields = line.split(";")
        if len(fields) < 3 or not fields[2].strip():
            continue
        try:
            count = float(fields[0].strip())
        except ValueError:
            count = None
        def number(index):
            try:
                return float(fields[index].strip().rstrip("%"))
            except (ValueError, IndexError):
                return None
        events.append({"event": fields[2].strip(), "count": count,
                       "runtime_raw": fields[3].strip() if len(fields) > 3 else None,
                       "running_percent": number(4), "unit": fields[1].strip(), "raw": line})
    return events


def run(args):
    build = read_build(args.build)
    if args.collector == "perf" and not sys.platform.startswith("linux"):
        raise ValueError("perf collection requires Linux")
    if args.collector == "perf" and build["instrumentation"] != "plain":
        raise ValueError("use plain binaries for perf counters; allocations alter instruction counts")
    out = new_result_dir(args.out)
    experiment = {"schema_version": 1, "build": build, "host": host(), "arguments": vars(args).copy(),
                  "notes": args.notes, "schedule": [], "runner_sha256": digest(__file__)}
    experiment["arguments"].pop("func")
    if args.collector == "perf":
        experiment["collector_environment"] = {"LC_ALL": "C"}
        experiment["perf_version"] = capture(["perf", "--version"])
        with open(out / "perf-list.txt", "w") as stream:
            command(["perf", "list"], stdout=stream)
    # Alternate AB/BA within every workload; reverse workload order each round.
    cases = [(case, motion, text, lines) for case in args.cases for motion in args.motions
             for text in args.texts for lines in args.lines]
    rows = []
    for repeat, (case, motion, text, lines), labels in paired_schedule(cases, args.repeats):
        for label in labels:
            name = f"{repeat:03}-{case}-{motion}-{text}-{lines}-{label}"
            argv = workload(build, label, case, motion, text, lines, args)
            if args.measure == "stages":
                argv += ["--samples", str(out / f"{name}.frames.json")]
            counter_path = out / f"{name}.perf.csv"
            if args.collector == "perf":
                argv = ["perf", "stat", "--no-big-num", "-x", ";", "-o", str(counter_path),
                        "-e", args.events, "--"] + argv
            job = {"name": name, "command": argv, "label": label, "repeat": repeat}
            experiment["schedule"].append(job)
            write_json(out / "experiment.json", experiment)
            env = dict(os.environ, LC_ALL="C") if args.collector == "perf" else None
            process = execute(argv, out / f"{name}.stdout.json", out / f"{name}.stderr.txt", args.cpu, env)
            result = json.loads((out / f"{name}.stdout.json").read_text())
            row = {**job, "result": result, "process": process,
                   "perf": perf_events(counter_path) if args.collector == "perf" else None}
            rows.append(row)
            with open(out / "runs.jsonl", "a") as stream:
                stream.write(json.dumps(row) + "\n")
            print(f"{name}: {result['elapsed_ns'] / result['frames'] / 1000:.2f} us/frame", flush=True)
            if args.cooldown:
                time.sleep(args.cooldown)
    verify_build(build)
    report_rows(rows, out / "summary.json", out / "summary.md", experiment)
    print(out / "summary.md")


def paired_schedule(cases, repeats):
    indexed_cases = list(enumerate(cases))
    for repeat in range(repeats):
        for index, case in (indexed_cases if repeat % 2 == 0 else reversed(indexed_cases)):
            labels = ("master", "fixed") if (repeat + index) % 2 == 0 else ("fixed", "master")
            yield repeat, case, labels


def quantile(values, fraction):
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    low = math.floor(position)
    high = math.ceil(position)
    return ordered[low] + (ordered[high] - ordered[low]) * (position - low)


def spread(values):
    return {"n": len(values), "median": statistics.median(values),
            "p25": quantile(values, 0.25), "p75": quantile(values, 0.75),
            "min": min(values), "max": max(values)}


def interpretation_notice(experiment):
    interpretation = experiment.get("interpretation", {})
    if interpretation.get("status") == "validation_only":
        return ["**Collection validation only.** " + interpretation["reason"],
                "Do not use timing or hardware-counter comparisons from this dataset as performance evidence.", ""]
    return []


def report_rows(rows, json_path, markdown_path, experiment):
    groups = {}
    for row in rows:
        r = row["result"]
        key = (r["case"], r["motion"], r["text"], r["lines"], r["measure"], r["qos"])
        groups.setdefault(key, {}).setdefault(row["label"], {})[row["repeat"]] = row
    summary = []
    table = ["# #389 CPU comparison", ""] + interpretation_notice(experiment) + [f"Collector: {experiment['arguments']['collector']}; "
             f"instrumentation: {experiment['build']['instrumentation']}.", "",
             "Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.",
             "Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.",
             "Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.", "",
             "| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |",
             "| --- | ---: | ---: | ---: | ---: |"]
    for key, labels in sorted(groups.items()):
        if set(labels) != {"master", "fixed"} or labels["master"].keys() != labels["fixed"].keys():
            raise ValueError(f"unpaired runs for {key}")
        entry = {"case": key[0], "motion": key[1], "text": key[2], "lines": key[3],
                 "measure": key[4], "qos": key[5], "variants": {}}
        values_by_label = {}
        for label, runs in labels.items():
            measures = {}
            values_by_label[label] = {}
            for repeat, row in runs.items():
                r = row["result"]
                ns = r["elapsed_ns"] / r["frames"]
                metrics = {"ns_per_frame": ns, "first_frame_ns": r["first_frame_ns"], "setup_ns": r["setup_ns"]}
                if r["glyphs_per_frame"]:
                    metrics["ns_per_glyph"] = ns / r["glyphs_per_frame"]
                if r["stages"]:
                    for stage, data in r["stages"].items():
                        metrics[f"{stage}_mean_ns"] = data["total_ns"] / r["frames"]
                        for percentile in ("p50_ns", "p95_ns", "p99_ns"):
                            metrics[f"{stage}_{percentile}"] = data[percentile]
                if r["allocations"]:
                    a = r["allocations"]
                    metrics.update({"alloc_calls_per_frame": a["calls"] / r["frames"],
                                    "alloc_bytes_per_frame": a["requested_bytes"] / r["frames"],
                                    "peak_live_bytes": a["peak_live_bytes"], "live_bytes_start": a["live_bytes_start"],
                                    "live_bytes_end": a["live_bytes_end"]})
                resource = row["process"]["resources"]
                if resource and resource["max_rss_bytes"] is not None:
                    metrics["process_peak_rss_bytes"] = resource["max_rss_bytes"]
                for event in row["perf"] or []:
                    if event["count"] is not None:
                        # Whole-process events: startup/warmup are included in numerator.
                        metrics[f"process_event_{event['event']}_per_measured_frame"] = event["count"] / r["frames"]
                for name, value in metrics.items():
                    measures.setdefault(name, []).append(value)
                values_by_label[label][repeat] = metrics
            entry["variants"][label] = {name: spread(v) for name, v in measures.items()}
        deltas = []
        for repeat in labels["master"]:
            a = labels["master"][repeat]["result"]
            b = labels["fixed"][repeat]["result"]
            if (a["glyphs_per_frame"], a["frames"]) != (b["glyphs_per_frame"], b["frames"]):
                raise ValueError("paired runs differ in work performed")
            deltas.append((b["elapsed_ns"] / a["elapsed_ns"] - 1) * 100)
        entry["paired_change_percent"] = spread(deltas)
        entry["paired_metrics"] = {}
        common = entry["variants"]["master"].keys() & entry["variants"]["fixed"].keys()
        # Percent changes with a zero baseline are undefined, not zero percent.
        for metric in common:
            pairs = [(values_by_label["master"][r][metric], values_by_label["fixed"][r][metric])
                     for r in labels["master"] if metric in values_by_label["master"][r]
                     and metric in values_by_label["fixed"][r]]
            entry["paired_metrics"][metric] = {
                "absolute_change": spread([b - a for a, b in pairs]),
                "percent_change": spread([(b / a - 1) * 100 for a, b in pairs]) if all(a for a, _ in pairs) else None,
            }
        summary.append(entry)
        change = entry["paired_change_percent"]
        table.append(f"| {' / '.join(map(str, key[:4]))} | "
                     f"{entry['variants']['master']['ns_per_frame']['median'] / 1000:.2f} | "
                     f"{entry['variants']['fixed']['ns_per_frame']['median'] / 1000:.2f} | "
                     f"{change['median']:+.2f}% | {change['p25']:+.2f}% to {change['p75']:+.2f}% |")
    table.extend(["", "Full stage, allocation, memory, and counter statistics are in `summary.json`. "
                  "Raw samples and commands are in `runs.jsonl` and `experiment.json`.", ""])
    write_json(json_path, summary)
    Path(markdown_path).write_text("\n".join(table))


def trace(args):
    if sys.platform != "darwin":
        raise ValueError("xctrace collection requires macOS")
    build = read_build(args.build)
    if build["instrumentation"] != "signposts":
        raise ValueError("prepare --instrumentation signposts for an identifiable measured interval")
    out = new_result_dir(args.out)
    experiment = {"schema_version": 1, "build": build, "host": host(), "arguments": vars(args).copy(),
                  "notes": args.notes, "schedule": [], "runner_sha256": digest(__file__)}
    experiment["arguments"].pop("func")
    # Save the actual options, not an assumed list of events for this Xcode/CPU.
    options_command = ["xcrun", "xctrace", "record", "--template", args.template, "--show-recording-options"]
    with open(out / "available-options.json", "w") as stream:
        command(options_command, stdout=stream)
    recording_options = None
    if args.recording_options:
        shutil.copyfile(args.recording_options, out / "recording-options.json")
        recording_options = out / "recording-options.json"
    elif args.counter_mode:
        options = json.loads((out / "available-options.json").read_text())
        counter = options.get("CPU Counters")
        if counter is None or "selectedCountingMode" not in counter:
            raise ValueError("this template/Xcode does not expose guided CPU counter modes")
        counter["selectedCountingMode"]["countingMode"] = args.counter_mode
        counter["selectedCountingModeDisplayName"] = {
            "bottlenecks": "CPU Bottlenecks", "processing": "Instruction Processing",
            "delivery": "Instruction Delivery"}[args.counter_mode]
        write_json(out / "recording-options.json", options)
        recording_options = out / "recording-options.json"
    records = []
    for repeat in range(args.repeats):
        for label in (("master", "fixed") if repeat % 2 == 0 else ("fixed", "master")):
            name = f"{repeat:03}-{label}"
            bundle = out / f"{name}.trace"
            target_stdout = out / f"{name}.workload.json"
            argv = ["xcrun", "xctrace", "record", "--template", args.template,
                    "--output", str(bundle), "--time-limit", f"{math.ceil(args.seconds + 30)}s", "--no-prompt",
                    "--target-stdout", str(target_stdout)]
            if recording_options:
                argv += ["--recording-options", str(recording_options)]
            argv += ["--launch", "--"] + workload(build, label, args.case, args.motion, args.text, args.lines, args, args.seconds)
            experiment["schedule"].append({"label": label, "repeat": repeat, "command": argv})
            write_json(out / "experiment.json", experiment)
            execute(argv, out / f"{name}.xctrace.stdout.txt", out / f"{name}.xctrace.stderr.txt")
            # Exit 0 isn't sufficient: a killed target or authorization error can yield an empty recording.
            target = json.loads(target_stdout.read_text())
            if not target["signposts"] or target["elapsed_ns"] < args.seconds * 1e9:
                raise ValueError("trace did not capture the complete signposted workload")
            exported = out / f"{name}-export"
            export_trace(bundle, exported)
            summary = json.loads((exported / "measured-summary.json").read_text())
            records.append({"label": label, "repeat": repeat, "workload": target, "counters": summary})
            print(bundle, flush=True)
    verify_build(build)
    write_json(out / "counter-runs.json", records)
    trace_report(records, out, experiment)


def export_trace(bundle, out):
    import xml.etree.ElementTree as ET
    out = Path(out)
    out.mkdir(parents=True, exist_ok=False)
    toc = out / "toc.xml"
    command(["xcrun", "xctrace", "export", "--input", bundle, "--toc", "--output", toc])
    root = ET.parse(toc).getroot()
    tables = []
    for run in root.findall("run"):
        number = run.attrib["number"]
        occurrences = {}
        for table in run.findall("./data/table"):
            schema = table.attrib.get("schema", "")
            occurrences[schema] = occurrences.get(schema, 0) + 1
            # Export metadata, signposts, scheduling, and counters. Stack samples and
            # processor instruction streams remain in the bundle to avoid huge XML files.
            if schema in {"OSSignpostIntervals", "CounterMetricByThread", "MetricTable", "ThreadQoSTable",
                          "process-info", "thread-info", "kdebug-counters-with-time-sample"}:
                if "'" in schema:
                    raise ValueError("unexpected quote in xctrace schema name")
                # Descriptions and swift-table attributes may contain quotes and
                # entire internal object dumps. Select duplicate schemas by ordinal.
                xpath = f"/trace-toc/run[@number='{number}']/data/table[@schema='{schema}'][{occurrences[schema]}]"
                destination = out / f"{len(tables):03}-{schema}.xml"
                command(["xcrun", "xctrace", "export", "--input", bundle,
                         "--xpath", xpath, "--output", destination])
                tables.append({"run": number, "attributes": table.attrib, "xpath": xpath,
                               "file": destination.name})
    write_json(out / "tables.json", tables)
    summarize_trace(out)


def table_rows(path):
    """Resolve xctrace's local XML references using column mnemonics, not positions."""
    import xml.etree.ElementTree as ET
    root = ET.parse(path).getroot()
    references = {e.attrib["id"]: e for e in root.iter() if "id" in e.attrib}
    def resolve(element):
        seen = set()
        while element is not None and "ref" in element.attrib:
            key = element.attrib["ref"]
            if key in seen:
                raise ValueError("cycle in xctrace references")
            seen.add(key)
            element = references[key]
        return element
    def nested(element, tag):
        element = resolve(element)
        if element is None:
            return None
        if element.tag == tag:
            return element.text
        for child in element:
            result = nested(child, tag)
            if result is not None:
                return result
        return None
    def cell(element):
        element = resolve(element)
        if element is None or element.tag == "sentinel":
            return None
        value = {"raw": element.text, "formatted": element.attrib.get("fmt"), "type": element.tag}
        if element.tag in ("process", "thread"):
            value["pid"] = nested(element, "pid")
            value["tid"] = nested(element, "tid")
        return value
    columns = [col.findtext("mnemonic") for col in root.findall(".//schema/col")]
    return [{name: cell(value) for name, value in zip(columns, row)} for row in root.findall(".//row")]


def summarize_trace(out):
    tables = json.loads((out / "tables.json").read_text())
    toc = out / "toc.xml"
    import xml.etree.ElementTree as ET
    target = ET.parse(toc).find("./run/info/target/process")
    if target is None:
        return
    pid = target.attrib["pid"]
    by_schema = {t["attributes"]["schema"]: out / t["file"] for t in tables}
    if "OSSignpostIntervals" not in by_schema:
        return
    intervals = table_rows(by_schema["OSSignpostIntervals"])
    intervals = [r for r in intervals if r.get("name") and r["name"]["raw"] == "Measured frames"
                 and r.get("process") and r["process"]["pid"] == pid]
    if len(intervals) != 1:
        raise ValueError("expected exactly one target-process 'Measured frames' signpost interval")
    interval = intervals[0]
    start = int(interval["start"]["raw"])
    end = start + int(interval["duration"]["raw"])
    tid = interval["start-thread"]["tid"]
    summary = {"target_pid": pid, "target_tid": tid, "measured_start_ns": start, "measured_end_ns": end,
               "selection": "Fully contained sample intervals on the signposted target thread; boundary samples excluded.",
               "metrics": [], "core_active_ns": {}, "core_type_active_ns": {}, "boundary_rows_excluded": 0}
    def contained(row):
        process = row.get("process")
        thread = row.get("thread")
        if not process or process["pid"] != pid or not thread or thread["tid"] != tid:
            return False
        lo = int(row["timestamp"]["raw"])
        hi = lo + int(row["duration"]["raw"])
        if lo >= start and hi <= end:
            return True
        if lo < end and hi > start:
            summary["boundary_rows_excluded"] += 1
        return False
    metrics = {}
    if "MetricTable" in by_schema:
        selected = []
        for row in table_rows(by_schema["MetricTable"]):
            if not contained(row):
                continue
            name = row["metric-display-name"]["raw"]
            event = row["pmi-event"]["raw"]
            core = row["core"]["formatted"]
            ratio = row["is-ratio"]["raw"] == "1"
            value = float(row["metric-value"]["raw"])
            duration = int(row["duration"]["raw"])
            record = {"metric": name, "event": event, "core": core, "is_ratio": ratio,
                      "value": value, "duration_ns": duration, "start_ns": int(row["timestamp"]["raw"])}
            selected.append(record)
            key = (name, event, core, ratio)
            acc = metrics.setdefault(key, {"sum": 0.0, "weighted_sum": 0.0, "sample_duration_ns": 0, "samples": 0})
            acc["sum"] += value
            acc["weighted_sum"] += value * duration
            acc["sample_duration_ns"] += duration
            acc["samples"] += 1
        with open(out / "measured-metrics.jsonl", "w") as stream:
            for row in selected:
                stream.write(json.dumps(row) + "\n")
    for (name, event, core, ratio), acc in sorted(metrics.items()):
        summary["metrics"].append({"metric": name, "event": event, "core": core, "is_ratio": ratio,
                                   "value": acc["weighted_sum"] / acc["sample_duration_ns"] if ratio else acc["sum"],
                                   "aggregation": "duration_weighted_mean" if ratio else "sum",
                                   "sample_duration_ns": acc["sample_duration_ns"], "samples": acc["samples"]})
    if "CounterMetricByThread" in by_schema:
        for row in table_rows(by_schema["CounterMetricByThread"]):
            if not contained(row):
                continue
            core = row["core"]["formatted"]
            duration = int(row["duration"]["raw"])
            summary["core_active_ns"][core] = summary["core_active_ns"].get(core, 0) + duration
            core_type = "Efficiency" if "(E Core)" in core else "Performance" if "(P Core)" in core else "Unknown"
            summary["core_type_active_ns"][core_type] = summary["core_type_active_ns"].get(core_type, 0) + duration
    write_json(out / "measured-summary.json", summary)


def trace_report(records, out, experiment=None):
    comparison = {}
    for record in records:
        workload = record["workload"]
        for m in record["counters"]["metrics"]:
            core_type = "Efficiency" if "(E Core)" in m["core"] else "Performance" if "(P Core)" in m["core"] else "Unknown"
            key = (m["metric"], m["event"], core_type, m["is_ratio"])
            variants = comparison.setdefault(key, {})
            entry = variants.setdefault(record["label"], {}).setdefault(record["repeat"],
                {"sum": 0.0, "duration_ns": 0, "frames": workload["frames"], "glyphs_per_frame": workload["glyphs_per_frame"]})
            entry["sum"] += m["value"] * m["sample_duration_ns"] if m["is_ratio"] else m["value"]
            entry["duration_ns"] += m["sample_duration_ns"]
    result = []
    table = ["# Instruments measured-interval comparison", ""] + interpretation_notice(experiment or {}) + [
             "Only fully contained samples on the signposted target thread are included. Boundary samples are excluded.",
             "Ratios are duration-weighted means reported by Instruments; counts are normalized per measured frame.",
             "Core types are taken from exported core descriptions. Small core residency gives weak comparisons.", "",
             "| Metric | Core type | Base | Fixed | Units |",
             "| --- | --- | ---: | ---: | --- |"]
    for (metric, event, core_type, ratio), variants in sorted(comparison.items()):
        data = {"metric": metric, "event": event, "core_type": core_type, "variants": {},
                "unit": "ratio" if ratio else "count_per_measured_frame"}
        for label, runs in variants.items():
            values = [v["sum"] / v["duration_ns"] if ratio else v["sum"] / v["frames"] for v in runs.values()]
            data["variants"][label] = spread(values)
        result.append(data)
        def display(label):
            if label not in data["variants"]:
                return "—"
            value = data["variants"][label]["median"]
            return f"{value * 100:.2f}%" if ratio else f"{value:.1f}"
        table.append(f"| {metric} | {core_type} | {display('master')} | {display('fixed')} | {data['unit']} |")
    table.extend(["", "See `counter-runs.json` for each trace's frame count, actual core residency, and sampled durations.", ""])
    write_json(out / "counter-summary.json", result)
    (out / "counter-summary.md").write_text("\n".join(table))


def mark_validation(args):
    out = Path(args.directory).resolve()
    experiment = json.loads((out / "experiment.json").read_text())
    experiment["interpretation"] = {
        "status": "validation_only", "reason": args.reason,
        "marked_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
    }
    write_json(out / "experiment.json", experiment)
    if (out / "runs.jsonl").exists():
        rows = [json.loads(line) for line in (out / "runs.jsonl").read_text().splitlines()]
        report_rows(rows, out / "summary.json", out / "summary.md", experiment)
    if (out / "counter-runs.json").exists():
        trace_report(json.loads((out / "counter-runs.json").read_text()), out, experiment)
    print(f"{out}: marked as collection validation only")


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest="action", required=True)
    prep = sub.add_parser("prepare", help="build pinned, dependency-matched executables")
    prep.add_argument("--base-source", default="https://github.com/femtovg/femtovg")
    prep.add_argument("--fixed-source", default="https://github.com/jfarmer/femtovg")
    prep.add_argument("--base-rev", default=BASE)
    prep.add_argument("--fixed-rev", default=FIXED)
    prep.add_argument("--font")
    prep.add_argument("--out")
    prep.add_argument("--cargo-target-dir", help="optional shared Cargo cache; published binaries remain separate")
    prep.add_argument("--offline", action="store_true", help="Cargo only; Git may fetch missing revisions")
    prep.add_argument("--instrumentation", choices=("plain", "allocations", "signposts"), default="plain")
    prep.set_defaults(func=prepare)
    for action, function in (("run", run), ("trace", trace)):
        child = sub.add_parser(action)
        child.add_argument("--build", default=str(ROOT / "target/profiling/plain/build.json"))
        child.add_argument("--out", required=True, help="new result directory")
        child.add_argument("--frames", type=int, default=3000)
        child.add_argument("--warmup", type=int, default=300)
        child.add_argument("--repeats", type=int, default=7 if action == "run" else 1)
        child.add_argument("--qos", choices=("default", "background"), default="default")
        child.add_argument("--notes", default="", help="power/thermal conditions and other experiment context")
        if action == "run":
            child.add_argument("--cases", type=lambda v: selection(v, CASES), default=list(CASES))
            child.add_argument("--motions", type=lambda v: selection(v, ("stationary", "moving")), default=["stationary", "moving"])
            child.add_argument("--texts", type=lambda v: selection(v, ("alphabet", "sentence", "diverse")), default=["sentence"])
            child.add_argument("--lines", type=lambda v: [int(n) for n in v.split(",")], default=[1])
            child.add_argument("--measure", choices=("bulk", "stages"), default="bulk")
            child.add_argument("--collector", choices=("none", "perf"), default="none")
            child.add_argument("--events", default="cycles,instructions,branches,branch-misses")
            child.add_argument("--cpu", type=int, help="Linux CPU affinity; applied to child/perf and inherited by workload")
            child.add_argument("--cooldown", type=float, default=0.0)
        else:
            child.add_argument("--case", choices=CASES, default="gradient-fill")
            child.add_argument("--motion", choices=("stationary", "moving"), default="stationary")
            child.add_argument("--text", choices=("alphabet", "sentence", "diverse"), default="sentence")
            child.add_argument("--lines", type=int, default=1)
            child.add_argument("--template", default="CPU Counters", help="installed template name or saved .tracetemplate")
            child.add_argument("--recording-options", help="JSON options supported by this Xcode installation")
            child.add_argument("--counter-mode", choices=("bottlenecks", "processing", "delivery"),
                               help="guided mode using the installed CPU Counters configuration")
            child.add_argument("--seconds", type=float, default=8.0)
            child.set_defaults(measure="bulk")
        child.set_defaults(func=function)
    export = sub.add_parser("export", help="export metadata/counters from an existing Instruments bundle")
    export.add_argument("trace")
    export.add_argument("--out", required=True)
    export.set_defaults(func=lambda args: export_trace(Path(args.trace).resolve(), Path(args.out).resolve()))
    report = sub.add_parser("report", help="regenerate summaries from raw runs")
    report.add_argument("directory")
    def regenerate(args):
        out = Path(args.directory)
        rows = [json.loads(line) for line in (out / "runs.jsonl").read_text().splitlines()]
        report_rows(rows, out / "summary.json", out / "summary.md", json.loads((out / "experiment.json").read_text()))
    report.set_defaults(func=regenerate)
    mark = sub.add_parser("mark-validation", help="record known interference without deleting raw samples")
    mark.add_argument("directory")
    mark.add_argument("--reason", required=True)
    mark.set_defaults(func=mark_validation)
    return p


def main():
    p = parser()
    args = p.parse_args()
    if args.action in ("run", "trace"):
        lines = args.lines if isinstance(args.lines, list) else [args.lines]
        if args.frames < 1 or args.warmup < 0 or args.repeats < 1 or any(n < 1 or n > 64 for n in lines) or len(set(lines)) != len(lines):
            p.error("frames/repeats >= 1, warmup >= 0, lines in 1..64")
        if args.qos == "background" and sys.platform != "darwin":
            p.error("background QoS is a macOS hint, not a portable affinity setting")
        if args.action == "trace" and (not math.isfinite(args.seconds) or args.seconds <= 0):
            p.error("seconds must be finite and positive")
        if args.action == "trace" and args.counter_mode and args.recording_options:
            p.error("use either --counter-mode or --recording-options")
        if args.action == "run" and (not math.isfinite(args.cooldown) or args.cooldown < 0):
            p.error("cooldown must be finite and nonnegative")
    try:
        args.func(args)
    except (ValueError, RuntimeError, OSError, subprocess.CalledProcessError) as error:
        if isinstance(error, subprocess.CalledProcessError) and error.stdout:
            for line in error.stdout.splitlines():
                try:
                    diagnostic = json.loads(line).get("message", {}).get("rendered")
                except (ValueError, AttributeError):
                    diagnostic = line
                if diagnostic:
                    print(diagnostic, file=sys.stderr)
        p.exit(1, f"profile.py: {error}\n")


if __name__ == "__main__":
    main()
