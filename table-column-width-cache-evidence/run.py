#!/usr/bin/env python3
"""Alternate frozen binaries; preserve every per-frame sample and process output."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

OUT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--suite', choices=['frames', 'large', 'memory', 'lifecycle', 'smoke'], default='frames')
parser.add_argument('--rounds', type=int, default=8)
parser.add_argument('--cpu', type=int, default=4)
parser.add_argument('--frames', type=int, default=80)
parser.add_argument('--output-dir', type=Path, required=True,
                    help='Fresh output directory; recorded evidence is never overwritten.')
args = parser.parse_args()
DEST = args.output_dir.resolve()
dest = DEST / f'{args.suite}.jsonl'
if dest.exists():
    raise SystemExit(f'Refusing to overwrite recorded evidence: {dest}')
logs = DEST / 'raw' / args.suite
logs.mkdir(parents=True, exist_ok=True)
binary_hashes = {v: hashlib.sha256(OUT.joinpath('bin', v).read_bytes()).hexdigest()
                 for v in ['baseline', 'patched']}

cases = [(case, mode) for case in ['small','medium','large','long','many']
         for mode in ['redraw', 'scroll'] if (case,mode) != ('small','scroll')]
cases += [('medium', 'resize'), ('medium','typography'), ('medium','replace'), ('wrap','redraw')]
if args.suite == 'smoke':
    cases = [('small', 'redraw')]
elif args.suite == 'large':
    cases = [('large', 'redraw'), ('large', 'scroll')]
elif args.suite == 'memory':
    cases = [(str(tables), str(cols)) for tables, cols in [(10,4),(100,4),(1000,4),(20000,1),(20000,4),(1000,16)]]
elif args.suite == 'lifecycle':
    cases = [('lifecycle', 'lifecycle')]

with dest.open('w') as result:
    for index, (case, mode) in enumerate(cases):
        for run in range(args.rounds):
            variants = ['baseline','patched'] if run % 2 == 0 else ['patched','baseline']
            if args.suite == 'lifecycle':
                variants = ['patched']
            for variant in variants:
                env = os.environ.copy()
                env.update({'WIDTH_CASE':case,'WIDTH_MODE':mode,'WIDTH_FRAMES':str(args.frames),
                            'WIDTH_TABLES':case,'WIDTH_COLS':mode,'MALLOC_ARENA_MAX':'1'})
                test = {'memory':'retained_tables','lifecycle':'cache_lifecycle'}.get(args.suite,'frames')
                command = ['taskset','-c',str(args.cpu),str(OUT/'bin'/variant),
                           f'text::node::column_width_evidence::{test}', '--exact', '--ignored', '--nocapture', '--test-threads=1']
                started = time.time()
                completed = subprocess.run(command, env=env, capture_output=True, text=True, timeout=240)
                prefix = logs / f'{case}-{mode}-{run:02d}-{variant}'
                prefix.with_suffix('.stdout').write_text(completed.stdout)
                prefix.with_suffix('.stderr').write_text(completed.stderr)
                if completed.returncode:
                    raise RuntimeError(f'{command}: {completed.returncode}\n{completed.stdout}\n{completed.stderr}')
                payloads = [line.split('WIDTH_JSON ',1)[1] for line in completed.stdout.splitlines() if 'WIDTH_JSON ' in line]
                assert len(payloads) == 1, completed.stdout
                record = {'variant':variant,'run':run,'timestamp':started,'wall_seconds':time.time()-started,
                          'binary_sha256':binary_hashes[variant],
                          'cpu':args.cpu,'load':os.getloadavg(),'data':json.loads(payloads[0])}
                result.write(json.dumps(record)+'\n')
                result.flush()
            print(f'{args.suite} {index+1}/{len(cases)} {case}/{mode} pair {run+1}/{args.rounds}', flush=True)
