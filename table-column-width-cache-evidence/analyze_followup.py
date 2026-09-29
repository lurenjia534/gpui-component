#!/usr/bin/env python3
"""Summarize additional large-table or real-window samples without filtering."""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import random
from statistics import median

parser = argparse.ArgumentParser()
parser.add_argument('--large', type=Path)
parser.add_argument('--native', type=Path)
args = parser.parse_args()
if not args.large and not args.native:
    parser.error('Specify --large and/or --native results directories')


def quantile(values, q):
    values = sorted(values)
    at = (len(values) - 1) * q
    low = int(at)
    return values[low] + (values[min(low + 1, len(values) - 1)] - values[low]) * (at - low)


def compare(before, after):
    assert len(before) == len(after) and len(before) >= 2
    rng = random.Random(534)
    differences = []
    for _ in range(10000):
        indices = [rng.randrange(len(before)) for _ in before]
        differences.append(median([before[i] for i in indices]) - median([after[i] for i in indices]))
    return {'before': median(before), 'after': median(after),
            'saved': median(before) - median(after),
            'saved_percent': 100 * (1 - median(after) / median(before)),
            'saved_ci95': [quantile(differences, .025), quantile(differences, .975)],
            'faster_pairs': sum(b > p for b, p in zip(before, after)),
            'before_process_range': [min(before), max(before)],
            'after_process_range': [min(after), max(after)]}


def groups(directory, name):
    grouped = defaultdict(dict)
    records = [json.loads(line) for line in (directory / f'{name}.jsonl').read_text().splitlines()]
    for record in records:
        data = record['data']
        grouped[data['case'], data['mode']][record['variant'], record['run']] = record
    for (case, mode), values in grouped.items():
        runs = sorted({run for variant, run in values})
        before = [values['baseline', run] for run in runs]
        after = [values['patched', run] for run in runs]
        yield {'case': case, 'mode': mode, 'pairs': len(runs)}, before, after


if args.large:
    summary = []
    for row, before, after in groups(args.large, 'large'):
        data = {'baseline': [r['data'] for r in before], 'patched': [r['data'] for r in after]}
        assert len({json.dumps(d['geometry'], sort_keys=True) for v in data.values() for d in v}) == 1
        row['draw_us'] = compare([median(d['samples_us']) for d in data['baseline']],
                                 [median(d['samples_us']) for d in data['patched']])
        for variant, values in data.items():
            per_process = []
            for d in values:
                samples = d['samples_us']
                quarter = len(samples) // 4
                per_process.append({
                    'median_us': median(samples), 'p95_us': quantile(samples, .95),
                    'p99_us': quantile(samples, .99),
                    'first_quarter_us': median(samples[:quarter]),
                    'last_quarter_us': median(samples[-quarter:]),
                    'over_16_667_us_percent': 100 * sum(x > 16667 for x in samples) / len(samples),
                })
            row[variant] = {'processes': per_process,
                            'p95_us': median([v['p95_us'] for v in per_process]),
                            'p99_us': median([v['p99_us'] for v in per_process]),
                            'first_quarter_us': median([v['first_quarter_us'] for v in per_process]),
                            'last_quarter_us': median([v['last_quarter_us'] for v in per_process]),
                            'timed_frames': sum(len(d['samples_us']) for d in values)}
        summary.append(row)
    (args.large / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    for row in summary:
        print(row['mode'], json.dumps(row['draw_us']))
        print('P95', row['baseline']['p95_us'], '->', row['patched']['p95_us'])

if args.native:
    summary = []
    for row, before, after in groups(args.native, 'native'):
        row['all_windows_active'] = all(r['valid_active_window'] for r in before + after)
        row['present_fps'] = compare([r['data']['present_fps'] for r in before],
                                     [r['data']['present_fps'] for r in after])
        # For FPS, increasing is better. Keep explicit units/sign semantics.
        fps = row['present_fps']
        row['fps_increase'] = -fps['saved']
        row['fps_increase_ci95'] = [-fps['saved_ci95'][1], -fps['saved_ci95'][0]]
        row['fps_increase_percent'] = 100 * (fps['after'] / fps['before'] - 1)
        row['fps_higher_pairs'] = sum(p['data']['present_fps'] > b['data']['present_fps']
                                      for b, p in zip(before, after))
        # These names describe lower-is-better timings, not rates.
        for key in ['saved', 'saved_percent', 'saved_ci95', 'faster_pairs']:
            del fps[key]
        row['all_hardware_rendered'] = all(
            r['data']['gpu'] is not None and not r['data']['gpu']['is_software_emulated']
            for r in before + after)
        for key in ['window_bounds', 'scale_factor', 'gpu', 'source_bytes', 'declared_refresh_hz']:
            assert len({json.dumps(r['data'][key], sort_keys=True) for r in before + after}) == 1, key
        row['environment'] = {key: before[0]['data'][key] for key in
                              ['window_bounds', 'scale_factor', 'gpu', 'source_bytes', 'declared_refresh_hz']}
        row['draw_ms'] = compare([median(r['data']['draw_ms']) for r in before],
                                 [median(r['data']['draw_ms']) for r in after])
        for variant, records in [('baseline', before), ('patched', after)]:
            values = [r['data'] for r in records]
            row[variant] = {
                'draw_p95_ms': median([quantile(d['draw_ms'], .95) for d in values]),
                'draw_p99_ms': median([quantile(d['draw_ms'], .99) for d in values]),
                'submit_p50_ms': median([median(d['submit_ms']) for d in values]),
                'submit_p95_ms': median([quantile(d['submit_ms'], .95) for d in values]),
                'interval_p50_ms': median([median(d['present_intervals_ms']) for d in values]),
                'interval_p95_ms': median([quantile(d['present_intervals_ms'], .95) for d in values]),
                'interval_p99_ms': median([quantile(d['present_intervals_ms'], .99) for d in values]),
                'long_interval_percent': median([
                    100 * sum(x > 1.5 * 1000 / d['declared_refresh_hz'] for x in d['present_intervals_ms'])
                    / len(d['present_intervals_ms']) for d in values]),
                'total_presents': sum(len(d['present_timestamps_ms']) for d in values),
            }
        summary.append(row)
    (args.native / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))
