#!/usr/bin/env python3
"""Build and run frozen native-window variants, preserving original evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

OUT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
sub = parser.add_subparsers(dest='action', required=True)
build = sub.add_parser('build')
build.add_argument('--work', type=Path, required=True)
run = sub.add_parser('run')
run.add_argument('--refresh-hz', type=float, required=True)
run.add_argument('--rounds', type=int, default=8)
run.add_argument('--presents', type=int, default=600)
run.add_argument('--cpu', type=int, help='Optional CPU pin; omit for normal application scheduling.')
run.add_argument('--output-dir', type=Path, required=True)
run.add_argument('--case', choices=['medium', 'large', 'many', 'wrap'])
run.add_argument('--mode', choices=['redraw', 'scroll'])
args = parser.parse_args()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


if args.action == 'build':
    work = args.work.resolve()
    subprocess.run([sys.executable, str(OUT / 'prepare.py'), '--work', str(work)], check=True)
    probe = work / 'crates/base/examples/column_width_native.rs'
    shutil.copyfile(OUT / 'native_probe.rs', probe)
    for variant in ['baseline', 'patched']:
        shutil.copyfile(OUT / f'{variant}-node.rs', work / 'crates/base/src/text/node.rs')
        command = ['cargo', 'build', '--offline', '--locked', '--release', '-p', 'gpui-base',
                   '--example', 'column_width_native', '--message-format=json', '-j', '12']
        with OUT.joinpath(f'native-{variant}-build.log').open('w') as stderr:
            result = subprocess.run(command, cwd=work, capture_output=False,
                                    stdout=subprocess.PIPE, stderr=stderr, text=True)
        OUT.joinpath(f'native-{variant}-build.jsonl').write_text(result.stdout)
        if result.returncode:
            for line in result.stdout.splitlines():
                event = json.loads(line)
                if event.get('reason') == 'compiler-message':
                    print(event['message']['rendered'])
            print(OUT.joinpath(f'native-{variant}-build.log').read_text()[-4000:])
            sys.exit(result.returncode)
        executable = None
        for line in result.stdout.splitlines():
            event = json.loads(line)
            if event.get('executable') and event.get('target', {}).get('name') == 'column_width_native':
                executable = Path(event['executable'])
        assert executable is not None
        binary = OUT / 'bin' / f'native-{variant}'
        binary.parent.mkdir(exist_ok=True)
        shutil.copyfile(executable, binary)
        binary.chmod(0o755)
        OUT.joinpath(f'native-{variant}-build-metadata.json').write_text(json.dumps({
            'command': command, 'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
            'sha256': {name: digest(OUT / name) for name in
                       [f'{variant}-node.rs', 'native_probe.rs', 'Cargo.lock', f'bin/native-{variant}']},
        }, indent=2) + '\n')
        print(binary, flush=True)
    sys.exit(0)

assert args.refresh_hz > 0 and args.rounds > 0 and args.presents >= 2
dest = args.output_dir.resolve()
dest.mkdir(parents=True, exist_ok=True)
records = dest / 'native.jsonl'
if records.exists():
    raise SystemExit(f'Refusing to overwrite {records}')
logs = dest / 'raw'
logs.mkdir(exist_ok=True)
hashes = {v: digest(OUT / 'bin' / f'native-{v}') for v in ['baseline', 'patched']}
with records.open('w') as output:
    cases = [('medium', 'scroll'), ('large', 'redraw'), ('large', 'scroll'), ('wrap', 'redraw')]
    if args.case or args.mode:
        cases = [(case, mode) for case in ([args.case] if args.case else ['medium', 'large', 'many', 'wrap'])
                 for mode in ([args.mode] if args.mode else ['redraw', 'scroll'])]
    for case, mode in cases:
        for run_ix in range(args.rounds):
            variants = ['baseline', 'patched'] if run_ix % 2 == 0 else ['patched', 'baseline']
            for variant in variants:
                env = os.environ.copy()
                env.update({'WIDTH_CASE': case, 'WIDTH_MODE': mode,
                            'WIDTH_PRESENTS': str(args.presents),
                            'WIDTH_REFRESH_HZ': str(args.refresh_hz), 'MALLOC_ARENA_MAX': '1'})
                command = [str(OUT / 'bin' / f'native-{variant}')]
                if args.cpu is not None:
                    command = ['taskset', '-c', str(args.cpu), *command]
                started = time.time()
                result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=240)
                prefix = logs / f'{case}-{mode}-{run_ix:02d}-{variant}'
                prefix.with_suffix('.stdout').write_text(result.stdout)
                prefix.with_suffix('.stderr').write_text(result.stderr)
                if result.returncode:
                    raise SystemExit(f'Native run failed ({result.returncode}); see {prefix}.stderr')
                payloads = [line.split('NATIVE_JSON ', 1)[1] for line in result.stdout.splitlines()
                            if 'NATIVE_JSON ' in line]
                assert len(payloads) == 1, result.stdout
                data = json.loads(payloads[0])
                record = {'variant': variant, 'run': run_ix, 'timestamp': started,
                          'cpu': args.cpu, 'allowed_cpus': sorted(os.sched_getaffinity(0)),
                          'binary_sha256': hashes[variant], 'data': data,
                          'valid_active_window': data['inactive_samples'] == 0}
                output.write(json.dumps(record) + '\n')
                output.flush()
                if not record['valid_active_window']:
                    raise SystemExit('Window lost focus; samples retained but invalid. Keep the window active and rerun in a fresh output directory.')
            print(f'{case}/{mode}: pair {run_ix + 1}/{args.rounds}', flush=True)
