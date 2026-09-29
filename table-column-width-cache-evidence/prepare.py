#!/usr/bin/env python3
"""Restore the frozen benchmark inputs without changing the working tree."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parent.parent
OUT = Path(__file__).resolve().parent
NODE = 'crates/base/src/text/node.rs'
parser = argparse.ArgumentParser()
parser.add_argument('--work', type=Path, default=Path('/tmp/gpui-column-evidence-20260929/work'))
args = parser.parse_args()
WORK = args.work.resolve()
if WORK == ROOT or ROOT in WORK.parents:
    raise SystemExit('Use a separate temporary directory, outside the source checkout.')

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args])

metadata = json.loads(OUT.joinpath('source-metadata.json').read_text())
for name, expected in metadata['files'].items():
    actual = hashlib.sha256(OUT.joinpath(name).read_bytes()).hexdigest()
    assert actual == expected, f'Frozen input changed: {name}'
assert git('show', f"{metadata['commit']}:{NODE}") == OUT.joinpath('baseline-node.rs').read_bytes()
WORK.mkdir(parents=True, exist_ok=True)
with tarfile.open(fileobj=io.BytesIO(git('archive', metadata['commit']))) as archive:
    archive.extractall(WORK, filter='data')
WORK.joinpath('Cargo.lock').write_bytes(OUT.joinpath('Cargo.lock').read_bytes())
manifest = WORK / 'crates/base/Cargo.toml'
manifest.write_text(manifest.read_text().replace('[features]', '[features]\nwidth-cache-present = []'))
OUT.joinpath('local-work.json').write_text(json.dumps({'work': str(WORK)}) + '\n')
print(f"Restored {metadata['commit']} in {WORK}")
