#!/usr/bin/env python3
"""Combine all valid paired native runs, documenting focus exclusions explicitly."""
import argparse
from collections import defaultdict
import hashlib
import json
import math
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--input', type=Path, action='append', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = Path(__file__).resolve().parent


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


assert not (args.output / 'native.jsonl').exists(), 'Never overwrite recorded samples'
args.output.mkdir(parents=True, exist_ok=True)
combined = []
excluded = []
pair_counts = defaultdict(int)
inputs = []
for directory in args.input:
    directory = directory.resolve()
    path = directory / 'native.jsonl'
    inputs.append({'path': str(path.relative_to(out)), 'sha256': sha256(path)})
    grouped = defaultdict(dict)
    for line_no, line in enumerate(path.read_text().splitlines(), 1):
        record = json.loads(line)
        data = record['data']
        assert record['valid_active_window'] == (data['inactive_samples'] == 0)
        assert data['gpu'] and not data['gpu']['is_software_emulated']
        for field in ['draw_ms', 'submit_ms', 'present_timestamps_ms']:
            assert len(data[field]) == 600, (path, line_no, field)
        assert len(data['present_intervals_ms']) == 599
        assert data['warmup_presents'] == 120
        assert math.isclose(data['present_fps'], 599000 / data['present_timestamps_ms'][-1])
        assert math.isclose(sum(data['present_intervals_ms']), data['present_timestamps_ms'][-1])
        build = json.loads((out / f'native-{record["variant"]}-build-metadata.json').read_text())
        assert record['binary_sha256'] == build['sha256'][f'bin/native-{record["variant"]}']
        origin = {'path': str(path.relative_to(out)), 'line': line_no, 'run': record['run']}
        record['source_record'] = origin
        key = (data['case'], data['mode'], record['run'])
        assert record['variant'] not in grouped[key]
        grouped[key][record['variant']] = record
    for (case, mode, run), variants in grouped.items():
        invalid = any(not r['valid_active_window'] for r in variants.values())
        if invalid:
            for record in variants.values():
                excluded.append({**record['source_record'], 'variant': record['variant'],
                                 'case': case, 'mode': mode,
                                 'inactive_samples': record['data']['inactive_samples'],
                                 'reason': 'Pair contains a run that lost focus; original data retained'})
            continue
        assert set(variants) == {'baseline', 'patched'}, (directory, case, mode, run)
        for record in variants.values():
            record['run'] = pair_counts[case, mode]
            combined.append(record)
        pair_counts[case, mode] += 1

assert len(pair_counts) == 4 and all(count >= 8 for count in pair_counts.values())
geometry = defaultdict(set)
for record in combined:
    data = record['data']
    geometry[data['case'], data['mode']].add((data['window_bounds'], data['document_bounds'],
                                            data['scroll_max'], data['scale_factor']))
assert all(len(values) == 1 for values in geometry.values()), 'Final A/B geometry differs'
destination = args.output / 'native.jsonl'
destination.write_text(''.join(json.dumps(r) + '\n' for r in combined))
manifest = {
    'selection_rule': 'All completed active-window A/B pairs from supplied batches; no timing-based exclusions',
    'inputs': inputs, 'excluded_records': excluded,
    'included_processes': len(combined), 'measured_presents': 600 * len(combined),
    'measured_intervals': 599 * len(combined),
    'final_geometry_matches_by_case': True,
    'pairs': {f'{case}/{mode}': count for (case, mode), count in pair_counts.items()},
    'environment': '../native-gpu-environment.json',
    'native_jsonl_sha256': sha256(destination),
    'source_hashes': {name: sha256(out / name) for name in
                      ['assemble_native.py', 'analyze_followup.py', 'native_probe.rs', 'native.py',
                       'baseline-node.rs', 'patched-node.rs']},
    'binaries': {v: sha256(out / 'bin' / f'native-{v}') for v in ['baseline', 'patched']},
}
(args.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest, indent=2))
