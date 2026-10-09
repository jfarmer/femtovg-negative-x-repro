#!/usr/bin/env python3
"""Run a short, serial selection funnel on the Pi; restore its governor on exit."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import time

ROOT = Path('/root/femtovg-negative-x-repro')
OUT = ROOT / 'out/pi3-389-best-2026-10-09'
os.chdir(ROOT)
os.environ['PATH'] = '/root/.cargo/bin:' + os.environ['PATH']
OUT.mkdir(exist_ok=True)
governor = Path('/sys/devices/system/cpu/cpufreq/policy0/scaling_governor')
original = governor.read_text()


def telemetry(phase):
    row = dict(phase=phase, utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
               governor=governor.read_text().strip(),
               frequency_khz=int(Path('/sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq').read_text()),
               throttled=subprocess.check_output(['vcgencmd', 'get_throttled'], text=True).strip(),
               temperature=subprocess.check_output(['vcgencmd', 'measure_temp'], text=True).strip())
    with (OUT / 'telemetry.jsonl').open('a') as stream:
        stream.write(json.dumps(row) + '\n')
    print('TELEMETRY ' + json.dumps(row), flush=True)


def config(name, **options):
    result = dict(id=name, case='best-fill', draws=512, working_set=1, glyph_percent=100,
                  order='grouped', shape='triangle', paint='solid', aa=False,
                  seed=389, motion='stationary', batch=True, visible_percent=100,
                  font_size=100, rotation=0, layout='overlap')
    result.update(options)
    return result


def run(name, plan, repeats, seconds, collector='none'):
    plan_file = OUT / (name + '-plan.json')
    plan_file.write_text(json.dumps(plan, indent=2) + '\n')
    telemetry(name + '-start')
    subprocess.run([sys.executable, 'profiling/stress.py', '--plan', str(plan_file),
                    '--build', 'target/profiling/best/build.json', '--out', str(OUT / name),
                    '--repeats', str(repeats), '--target-seconds', str(seconds), '--warmup', '16',
                    '--cpu', '2', '--collector', collector,
                    '--notes', 'Best-case selection funnel. CPU2, performance governor. Serial measurements; no builds, packing or downloads during measurements.'], check=True)
    telemetry(name + '-end')
    return json.loads((OUT / name / 'summary.json').read_text())


if (OUT / 'scout-best').exists():
    raise RuntimeError('Refusing to overwrite an existing best-case experiment')
print('PHASE build', flush=True)
subprocess.run([sys.executable, 'profiling/profile.py', 'prepare', '--offline',
                '--out', 'target/profiling/best', '--cargo-target-dir', 'target/profiling/plain/cargo'], check=True)
shutil.copy2(ROOT / 'target/profiling/best/build.json', OUT / 'build-best.json')
shutil.copy2(ROOT / 'target/profiling/stress-barrier/build.json', OUT / 'build-existing.json')

plan = [
    config('visible-fill-unbatched', batch=False),
    config('visible-fill-batched'),
    config('visible-fill-batched-aa', aa=True),
    config('visible-stroke-batched', case='best-stroke'),
    config('offscreen-fill-batched', visible_percent=0),
    config('mostly-offscreen-fill-batched', visible_percent=10),
    config('mostly-offscreen-fill-grid', visible_percent=10, layout='grid'),
    config('visible-fill-grid', layout='grid'),
    config('rotated-labels-fill', font_size=18, rotation=0.12, layout='labels', aa=True),
    config('rotated-labels-stroke', case='best-stroke', font_size=18, rotation=0.12, layout='labels', aa=True),
    config('small-labels-atlas-control', font_size=18, layout='labels', aa=True),
]
try:
    governor.write_text('performance')
    print('PHASE scout-best', flush=True)
    scout = run('scout-best', plan, 3, 0.3)
    lookup = {c['id']: c for c in plan}
    ranked = sorted(scout, key=lambda row: row['paired_change_percent']['median'])
    chosen = []
    families = set()
    selection = []
    for row in ranked:
        cfg = lookup[row['case']]
        stats = row['paired_change_percent']
        if cfg['id'].endswith('control'):
            family = 'control'
        elif cfg['layout'] != 'overlap':
            family = 'spaced-scenes'
        elif cfg['visible_percent'] < 100:
            family = 'offscreen-overlap'
        else:
            family = 'visible-overlap'
        # With three pairs, a negative median means at least two pairs improved.
        promote = family != 'control' and family not in families and stats['median'] <= -1 and len(chosen) < 3
        selection.append(dict(id=row['case'], family=family, timing=stats, promoted=promote))
        if promote:
            chosen.append(cfg)
            families.add(family)
    (OUT / 'selection.json').write_text(json.dumps(selection, indent=2) + '\n')
    print('SELECTION ' + json.dumps(selection), flush=True)
    if chosen:
        print('PHASE confirm ' + ', '.join(c['id'] for c in chosen), flush=True)
        confirmed = run('confirm-best', chosen, 7, 1.0)
        winners = [r for r in sorted(confirmed, key=lambda r: r['paired_change_percent']['median'])
                   if r['paired_change_percent']['p75'] < 0][:2]
        if winners:
            print('PHASE core-counters', flush=True)
            run('perf-core-best', [lookup[r['case']] for r in winners], 3, 0.6, 'perf')
    (OUT / 'complete.json').write_text(json.dumps(dict(complete=True, utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))) + '\n')
finally:
    governor.write_text(original)
    telemetry('restored-' + original.strip())

print('PHASE archive (measurements finished)', flush=True)
with tarfile.open('/root/pi389-best-results.tar.gz', 'w:gz') as archive:
    archive.add(OUT, arcname=OUT.name, filter=lambda info: None if info.name.endswith('incoming.json') else info)
print('DONE', flush=True)
