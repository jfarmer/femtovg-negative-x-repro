"""Checks that scenario selection and paired reports preserve equivalent work."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import stress
import test_profile
import verify_stress


class StressTests(unittest.TestCase):
    def best_config(self):
        config = stress.initial_plan()[0].copy()
        config.update(id='best-example', case='best-fill', draws=17,
                      batch=False, visible_percent=33, font_size=18,
                      rotation=0.12, layout='labels')
        return config

    def best_result(self, config):
        return dict(glyphs_per_frame=config['draws'], stress=dict(
            draws=config['draws'], prepared_glyphs=config['draws'], unique_paths=0,
            glyph_percent=100, visible_glyph_count=config['draws'] * config['visible_percent'] // 100,
            **{field: config[field] for field in ('batch', 'visible_percent', 'font_size', 'rotation', 'layout', 'aa', 'paint')}))

    def test_best_argv_forwards_all_requested_options(self):
        config = self.best_config()
        build = dict(binaries={'master': {'path': '/bin/profile-master'}}, font='/font.ttf')
        argv = stress.argv_for(build, 'master', config, 32, 16, 'bulk')
        options = dict(zip(argv[1::2], argv[2::2]))
        for field in ('batch', 'visible_percent', 'font_size', 'rotation', 'layout'):
            self.assertEqual(options['--' + field.replace('_', '-')], str(config[field]).lower())
        old = stress.argv_for(build, 'master', stress.initial_plan()[0], 32, 16, 'bulk')
        self.assertNotIn('--batch', old)
        self.assertNotIn('--font-size', old)

    def test_best_work_validates_exact_count_and_visibility_rounding(self):
        config = self.best_config()
        result = self.best_result(config)
        self.assertEqual(result['stress']['visible_glyph_count'], 5)
        for case in ('best-fill', 'best-stroke'):
            config['case'] = case
            verify_stress.verify_work(result, config)

    def test_best_work_rejects_changed_requested_options(self):
        config = self.best_config()
        result = self.best_result(config)
        mutations = dict(batch=True, visible_percent=34, layout='grid', font_size=19,
                         rotation=0.13, prepared_glyphs=18, visible_glyph_count=6,
                         unique_paths=1)
        for field, value in mutations.items():
            with self.subTest(field=field):
                changed = copy.deepcopy(result)
                changed['stress'][field] = value
                with self.assertRaisesRegex(AssertionError, field):
                    verify_stress.verify_work(changed, config)

    def test_best_work_accepts_f32_decimal_rounding(self):
        config = self.best_config()
        config['rotation'] = 0.123456789
        result = self.best_result(config)
        result['stress']['rotation'] = 0.12345679
        verify_stress.verify_work(result, config)

    def test_plan_writer_creates_parent_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "new" / "plan.json"
            with patch("sys.argv", ["stress.py", "--write-plan", str(path)]):
                stress.main()
            self.assertEqual(len(json.loads(path.read_text())), 78)

    def test_plan_has_controls_and_valid_draw_counts(self):
        plan = stress.initial_plan()
        self.assertEqual(len(plan), 78)
        self.assertEqual(len({c['id'] for c in plan}), 78)
        for config in plan:
            if config['case'] == 'cache-glyph-fill':
                self.assertGreaterEqual(config['draws'], config['working_set'])
            if config['case'].startswith(('cache-', 'mixed-')):
                controls = [c for c in plan if all(c[k] == config[k] for k in config if k not in ('id', 'order'))]
                self.assertEqual({c['order'] for c in controls}, {'grouped', 'shuffled'})
            self.assertFalse(config['case'] == 'tiny-path-fill' and config['shape'] == 'line')

    def test_calibration_preserves_complete_motion_cycles(self):
        self.assertEqual(stress.frames_for(1e9, .6), 16)
        self.assertEqual(stress.frames_for(1e6, .6), 608)
        self.assertEqual(stress.frames_for(1, .6), 8192)
        for cost in (1, 2000, 1e6, 1e9):
            self.assertEqual(stress.frames_for(cost, .6) % 16, 0)

    def test_report_rejects_different_stress_work_even_with_equal_glyph_counts(self):
        rows = [test_profile.ProfilingTests().row('master', 1000), test_profile.ProfilingTests().row('fixed', 900)]
        for row in rows:
            row['result']['stress'] = dict(draws=1024, schedule_hash=123, aa=True)
        experiment = dict(arguments={'collector':'none'}, build={'instrumentation':'plain'})
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            stress.common.report_rows(rows,out/'summary.json',out/'summary.md',experiment)
            changed = copy.deepcopy(rows)
            changed[1]['result']['stress']['schedule_hash'] = 124
            with self.assertRaisesRegex(ValueError,'differ in work'):
                stress.common.report_rows(changed,out/'summary.json',out/'summary.md',experiment)


if __name__ == '__main__':
    unittest.main()
