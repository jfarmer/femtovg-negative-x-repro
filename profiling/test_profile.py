"""Checks that counter/report plumbing cannot silently change scientific meaning."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("repro_profile", Path(__file__).with_name("profile.py"))
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)


class ProfilingTests(unittest.TestCase):
    def test_linux_telemetry_keeps_units_and_missing_readings(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            zone = root / "class/thermal/thermal_zone0"
            zone.mkdir(parents=True)
            (zone / "type").write_text("cpu-thermal\n")
            (zone / "temp").write_text("42000\n")
            policy = root / "devices/system/cpu/cpu2/cpufreq"
            policy.mkdir(parents=True)
            (policy / "scaling_governor").write_text("performance\n")
            (policy / "scaling_cur_freq").write_text("1200000\n")
            (policy / "cpuinfo_cur_freq").write_text("unavailable\n")
            result = profile.linux_snapshot(root, cpu=2)
            self.assertEqual(result["thermal_zones"][0]["temperature_millicelsius"], 42000)
            self.assertEqual(result["cpu_frequency"]["cpu2"]["scaling_cur_freq_khz"], 1200000)
            self.assertEqual(result["cpu_frequency"]["cpu2"]["governor"], "performance")
            self.assertIsNone(result["cpu_frequency"]["cpu2"]["cpuinfo_cur_freq_khz"])
            self.assertIsNone(result["cpu_frequency"]["cpu2"]["scaling_min_freq_khz"])
            self.assertEqual(profile.linux_snapshot(root, cpu=3)["cpu_frequency"], {})

    def test_pair_order_alternates_even_when_workload_order_reverses(self):
        schedule = list(profile.paired_schedule(["a", "b"], 2))
        self.assertEqual(schedule, [(0, "a", ("master", "fixed")), (0, "b", ("fixed", "master")),
                                    (1, "b", ("master", "fixed")), (1, "a", ("fixed", "master"))])

    def test_unsupported_perf_events_are_missing_not_zero(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "perf.csv"
            path.write_text("# perf output\n1234;;cycles;100000;50.00;;\n"
                            "<not supported>;;L1-dcache-load-misses;0;0.00;;\n")
            events = profile.perf_events(path)
        self.assertEqual(events[0]["count"], 1234)
        self.assertEqual(events[0]["running_percent"], 50)
        self.assertIsNone(events[1]["count"])

    def row(self, label, elapsed, glyphs=30):
        return {"label": label, "repeat": 0, "result": {
            "case": "gradient-fill", "motion": "stationary", "text": "sentence", "lines": 1,
            "measure": "bulk", "qos": "default", "elapsed_ns": elapsed, "frames": 10,
            "glyphs_per_frame": glyphs, "first_frame_ns": 50, "setup_ns": 100,
            "stages": None, "allocations": {"calls": 0, "requested_bytes": 0,
            "peak_live_bytes": 100, "live_bytes_start": 100, "live_bytes_end": 100}},
            "process": {"resources": None}, "perf": None}

    def test_report_pairs_work_and_keeps_zero_baseline_ratios_undefined(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            experiment = {"arguments": {"collector": "none"}, "build": {"instrumentation": "allocations"}}
            profile.report_rows([self.row("master", 1000), self.row("fixed", 900)],
                                out / "summary.json", out / "summary.md", experiment)
            summary = json.loads((out / "summary.json").read_text())[0]
            self.assertAlmostEqual(summary["paired_change_percent"]["median"], -10)
            self.assertIsNone(summary["paired_metrics"]["alloc_calls_per_frame"]["percent_change"])
            with self.assertRaisesRegex(ValueError, "differ in work"):
                profile.report_rows([self.row("master", 1000), self.row("fixed", 900, glyphs=29)],
                                    out / "summary.json", out / "summary.md", experiment)
            with self.assertRaisesRegex(ValueError, "unpaired"):
                profile.report_rows([self.row("master", 1000)], out / "summary.json", out / "summary.md", experiment)

    def test_changed_executable_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            font, binary = out / "font", out / "binary"
            font.write_bytes(b"font")
            binary.write_bytes(b"binary")
            profile.write_json(out / "build.json", {"font": str(font), "font_sha256": profile.digest(font),
                "binaries": {"master": {"path": str(binary), "sha256": profile.digest(binary)}}})
            binary.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "artifact changed"):
                profile.read_build(out / "build.json")

    def test_report_regeneration_preserves_known_interference(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            experiment = {"arguments": {"collector": "none"}, "build": {"instrumentation": "plain"},
                          "interpretation": {"status": "validation_only", "reason": "Concurrent CPU-heavy build battery."}}
            profile.report_rows([self.row("master", 1000), self.row("fixed", 900)],
                                out / "summary.json", out / "summary.md", experiment)
            report = (out / "summary.md").read_text()
        self.assertIn("Collection validation only", report)
        self.assertIn("Concurrent CPU-heavy build battery", report)
        self.assertIn("Do not use timing or hardware-counter comparisons", report)

    def test_trace_references_boundaries_and_process_filter(self):
        # Includes a cross-boundary sample and another process on the same core.
        # Neither may contribute to the measured target's cycle count.
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            (out / "toc.xml").write_text('<trace-toc><run><info><target><process pid="42"/></target></info></run></trace-toc>')
            process = '<process id="p"><pid>42</pid></process>'
            thread = '<thread id="t"><tid>11</tid><process ref="p"/></thread>'
            def schema(names):
                return '<schema>' + ''.join(f'<col><mnemonic>{n}</mnemonic></col>' for n in names) + '</schema>'
            (out / "signposts.xml").write_text('<trace-query-result>' + schema(
                ["start", "duration", "name", "process", "start-thread"]) + '<row><start-time>100</start-time>'
                '<duration>100</duration><signpost-name>Measured frames</signpost-name>' + process + thread + '</row></trace-query-result>')
            def metric_row(start, duration, value, other=False):
                proc = '<process><pid>43</pid></process>' if other else '<process ref="p"/>'
                return f'<row><start-time>{start}</start-time><duration>{duration}</duration>' + proc + '<thread ref="t"/>' + (
                    '<string>Cycles</string><string>cycle</string><core fmt="CPU 4 (P Core)">4</core>'
                    f'<boolean>0</boolean><fixed-decimal>{value}</fixed-decimal></row>')
            (out / "metrics.xml").write_text('<trace-query-result>' + process + thread + schema([
                "timestamp", "duration", "process", "thread", "metric-display-name", "pmi-event", "core", "is-ratio", "metric-value"])
                + metric_row(90, 20, 9999) + metric_row(110, 20, 100) + metric_row(140, 40, 200)
                + metric_row(110, 20, 8888, other=True) + '</trace-query-result>')
            profile.write_json(out / "tables.json", [
                {"attributes": {"schema": "OSSignpostIntervals"}, "file": "signposts.xml"},
                {"attributes": {"schema": "MetricTable"}, "file": "metrics.xml"}])
            profile.summarize_trace(out)
            result = json.loads((out / "measured-summary.json").read_text())
        self.assertEqual(result["metrics"][0]["value"], 300)
        self.assertEqual(result["metrics"][0]["sample_duration_ns"], 60)
        self.assertEqual(result["boundary_rows_excluded"], 1)
        self.assertEqual(result["target_tid"], "11")


if __name__ == "__main__":
    unittest.main()
