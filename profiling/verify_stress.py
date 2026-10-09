"""Validate saved stress pairs, source provenance, stage arrays, and optional Linux conditions."""
import argparse
import hashlib
import json
import math
from pathlib import Path


def read_json(path):
    return json.loads(path.read_text())


def source_matches(directory, hashes):
    for name, expected in hashes.items():
        path = directory / Path(name).name
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            return False
    return True


def verify_work(result, config):
    """Check that recorded work matches the requested scenario, not just its paired variant."""
    stress = result['stress']
    assert stress['draws'] == config['draws'], 'draws'
    if config['case'] in ('best-fill', 'best-stroke'):
        glyphs = config['draws']
        expected = dict(batch=config.get('batch', True),
                        visible_percent=config.get('visible_percent', 100),
                        layout=config.get('layout', 'overlap'))
        for field, value in expected.items():
            assert stress[field] == value, field
        for field, default in (('font_size', 100.0), ('rotation', 0.0)):
            value = float(config.get(field, default))
            # Rust parses these options as f32 and prints the shortest round-tripping
            # decimal, which can differ slightly from the plan's Python float.
            assert math.isclose(stress[field], value, rel_tol=1e-7, abs_tol=1e-8), field
        assert stress['prepared_glyphs'] == glyphs, 'prepared_glyphs'
        assert stress['visible_glyph_count'] == glyphs * expected['visible_percent'] // 100, 'visible_glyph_count'
        assert stress['glyph_percent'] == 100, 'glyph_percent'
        for field in ('aa', 'paint'):
            assert stress[field] == config[field], field
    elif config['case'] == 'cache-glyph-fill':
        glyphs = config['draws']
        assert stress['unique_glyph_representations'] == config['working_set'], 'unique_glyph_representations'
    elif config['case'].startswith('mixed-'):
        glyphs = config['draws'] * config['glyph_percent'] // 100
    else:
        glyphs = 0
    assert result['glyphs_per_frame'] == glyphs, 'glyphs_per_frame'
    assert stress['unique_paths'] == config['draws'] - glyphs, 'unique_paths'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory')
    parser.add_argument('--governor')
    parser.add_argument('--frequency-khz', type=int)
    parser.add_argument('--require-perf-100', action='store_true')
    args = parser.parse_args()
    root = Path(args.directory)
    phases = sorted(path for path in root.iterdir() if path.is_dir() and (path / 'runs.jsonl').exists())
    assert phases, 'no stress experiments found'
    total = 0
    for path in phases:
        experiment = read_json(path / 'experiment.json')
        rows = [json.loads(line) for line in (path / 'runs.jsonl').read_text().splitlines()]
        expected = len(experiment['plan']) * experiment['arguments']['repeats'] * 2
        assert len(rows) == len(experiment['schedule']) == expected, (path.name, len(rows))
        candidates = (root / 'source-v1', root / 'source-v2', root / 'sources-existing',
                      root / 'sources-best', Path(__file__).parent)
        assert any(source_matches(d, experiment['build']['harness_sha256']) for d in candidates), path.name
        pairs = {}
        for row in rows:
            result, config = row['result'], row['config']
            raw = read_json(path / (row['name'] + '.stdout.json'))
            raw['workload_case'], raw['case'] = raw['case'], config['id']
            assert raw == result, row['name']
            pairs.setdefault((row['repeat'], config['id']), {})[row['label']] = result
            assert result['frames'] % 16 == 0
            verify_work(result, config)
            if row['perf'] and args.require_perf_100:
                assert all(e['count'] is not None and e['running_percent'] == 100 for e in row['perf'])
            if result['stages']:
                samples = read_json(path / (row['name'] + '.frames.json'))
                assert all(len(values) == result['frames'] for values in samples.values())
                triples = zip(samples['draw_ns'], samples['flush_ns'], samples['frame_ns'])
                assert all(draw + flush == frame for draw, flush, frame in triples)
                for stage in ('draw', 'flush', 'frame'):
                    assert sum(samples[stage + '_ns']) == result['stages'][stage]['total_ns']
            if args.governor or args.frequency_khz:
                for endpoint in ('environment_before', 'environment_after'):
                    snapshot = row['process'][endpoint]
                    assert snapshot and snapshot['cpu_frequency'], row['name']
                    for policy in snapshot['cpu_frequency'].values():
                        assert not args.governor or policy['governor'] == args.governor, row['name']
                        assert not args.frequency_khz or policy['scaling_cur_freq_khz'] == args.frequency_khz, row['name']
        for pair in pairs.values():
            assert set(pair) == {'master', 'fixed'}
            master, fixed = pair['master'], pair['fixed']
            for field in ('frames', 'glyphs_per_frame', 'stress'):
                assert master[field] == fixed[field], field
        status = experiment.get('interpretation', {}).get('status', 'performance')
        print(path.name, len(rows), 'children', len(pairs), 'checked pairs', status)
        total += len(rows)
    telemetry = root / 'telemetry.jsonl'
    if telemetry.exists():
        events = [json.loads(line) for line in telemetry.read_text().splitlines()]
        assert all(event['throttled'].strip() == 'throttled=0x0' for event in events)
        print('telemetry phases', len(events), 'all throttled=0x0')
    print('total checked children', total)


if __name__ == '__main__':
    main()
