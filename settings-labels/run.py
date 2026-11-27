"""Build pinned variants, replay the scene, or summarize archived timings."""
import argparse
import json
import os
from pathlib import Path
import shutil
from statistics import median
import subprocess

HERE = Path(__file__).resolve().parent
OUT = HERE.parent / "out/settings-labels"
BASE = "786d75778291fc579be9a013466e10ddd80a7884"
BITMAP = "4b7a9dc3baa4d9b14d97091c0c8846b09b639b85"
BOTH = "72b6ec573e92d1c8e5d346bd91033c1bcce8aa6d"
REVISIONS = {"base": BASE, "strikes": BITMAP, "colr": BASE, "both": BOTH}
CONFIGS = ("swash", "mixed")


def variants(backend, config):
    if backend == "void":
        return ["base", "colr"] if config == "mixed" else ["base", "colr", "both", "strikes"]
    return list(REVISIONS)


def build(backend):
    OUT.mkdir(parents=True, exist_ok=True)
    store = OUT / "objects"
    if not store.exists():
        subprocess.run(["git", "init", "--bare", str(store)], check=True)
    if subprocess.run(["git", "-C", store, "cat-file", "-e", BOTH], stderr=subprocess.DEVNULL).returncode:
        url = os.environ.get("FEMTOVG_URL", "https://github.com/jfarmer/femtovg.git")
        subprocess.run(["git", "-C", store, "fetch", "--no-tags", url, BOTH], check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", OUT / "target")).resolve()
    for tag, rev in REVISIONS.items():
        lib = OUT / tag / "femtovg"
        project = OUT / tag / "benchmark"
        project.mkdir(parents=True, exist_ok=True)
        if not lib.exists():
            subprocess.run(["git", "-C", store, "worktree", "add", "--detach", lib, rev], check=True)
        head = subprocess.check_output(["git", "-C", lib, "rev-parse", "HEAD"], text=True).strip()
        if head != rev:
            raise SystemExit(f"Unexpected revision in {lib}")
        if tag == "colr":
            patch = HERE / "colr-only.patch"
            applied = subprocess.run(["git", "-C", lib, "apply", "-R", "--check", patch], stderr=subprocess.DEVNULL)
            if applied.returncode:
                subprocess.run(["git", "-C", lib, "apply", patch], check=True)
        changed = subprocess.check_output(["git", "-C", lib, "diff", "--name-only", "HEAD"], text=True).splitlines()
        expected = ["src/text/font.rs"] if tag == "colr" else []
        if changed != expected:
            raise SystemExit(f"Unexpected changes in {lib}")
        if tag == "colr":
            blob = subprocess.check_output(["git", "-C", lib, "hash-object", "src/text/font.rs"], text=True).strip()
            if blob != "5a2bccd9db0b8f1ec50460f4b0aa20aaf27d6345":
                raise SystemExit("COLR-only source differs from the measured revision")
        shutil.copytree(HERE / "src", project / "src", dirs_exist_ok=True)
        shutil.copy(HERE / "Cargo.toml.in", project / "Cargo.toml")
        shutil.copy(HERE / "Cargo.lock", project / "Cargo.lock")
        env = dict(os.environ, CARGO_TARGET_DIR=str(target),
                   BENCH_FONT=str(lib / "examples/assets/RobotoFlex-VariableFont.ttf"))
        for config in CONFIGS:
            if tag not in variants(backend, config):
                continue
            features = []
            if backend == "metal":
                features.append("metal")
            if config == "mixed":
                features.append("mixed")
            cmd = ["cargo", "build", "--release", "--locked", "--bin", backend]
            if features:
                cmd.extend(["--features", ",".join(features)])
            subprocess.run(cmd, cwd=project, env=env, check=True)
            shutil.copy(target / "release" / backend, OUT / f"{backend}-{config}-{tag}")


def measure(backend, core):
    def sample(config, tag, frames, capture=False):
        cmd = [str(OUT / f"{backend}-{config}-{tag}"), str(frames)]
        path = OUT / f"{backend}-{config}-{tag}.rgba"
        if capture:
            cmd.append(str(path))
        if core is not None:
            cmd = ["taskset", "-c", str(core), *cmd]
        values = list(map(int, subprocess.check_output(cmd, text=True).split()))
        return path.read_bytes() if capture else values

    rows = {config: [] for config in CONFIGS}
    if backend != "void":
        for config in CONFIGS:
            images = [sample(config, tag, 4, True) for tag in variants(backend, config)]
            if len(set(images)) != 1 or len(set(images[0][::4])) < 20:
                raise SystemExit(f"{config}: pixel check failed")
    for i in range(16):
        for config in CONFIGS if i % 2 == 0 else reversed(CONFIGS):
            tags = variants(backend, config)
            offset = i % len(tags)
            order = tags[offset:] + tags[:offset]
            if i // len(tags) % 2:
                order.reverse()
            rows[config].append({tag: sample(config, tag, 201) for tag in order})
    (OUT / f"{backend}.json").write_text(json.dumps(rows) + "\n")
    return rows


def archived(backend):
    data = HERE / "results"
    if backend != "void":
        return json.loads((data / f"{backend}.json").read_text())
    rows = {config: [{} for _ in range(16)] for config in CONFIGS}
    for line in (data / "void.txt").read_text().splitlines():
        i, config, tag, ns = line.split()
        rows[config][int(i)][tag] = [int(ns)]
    return rows


def summarize(rows):
    print("| Features | Variant | Median CPU (ms/frame) | Paired change (%) |")
    print("|---|---|---:|---:|")
    for config, pairs in rows.items():
        for tag in pairs[0]:
            elapsed = median(row[tag][0] for row in pairs) / 1e6
            delta = median(row[tag][0] / row["base"][0] - 1 for row in pairs)
            print(f"| {config} | {tag} | {elapsed:.4f} | {delta:+.1%} |")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["build", "measure", "archived"])
    parser.add_argument("backend", choices=["metal", "opengl", "void"])
    parser.add_argument("--core", type=int, help="pin each process with Linux taskset")
    args = parser.parse_args()
    if args.action == "build":
        build(args.backend)
    else:
        summarize(archived(args.backend) if args.action == "archived" else measure(args.backend, args.core))
