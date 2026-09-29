#!/usr/bin/env python3
"""Build one frozen variant with the same evidence-only harness."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

OUT = Path(__file__).resolve().parent
meta = json.loads(OUT.joinpath('source-metadata.json').read_text())
local = OUT / 'local-work.json'
WORK = Path(json.loads(local.read_text())['work'] if local.exists() else meta['work'])
variant = sys.argv[1]
assert variant in ['baseline', 'patched']
node = OUT.joinpath(f'{variant}-node.rs').read_text()
node += '\n#[cfg(test)]\n#[path = "column_width_evidence.rs"]\nmod column_width_evidence;\n'
WORK.joinpath('crates/base/src/text/node.rs').write_text(node)
shutil.copyfile(OUT / 'frame_probe.rs', WORK / 'crates/base/src/text/column_width_evidence.rs')
cmd = ['cargo', 'test', '--offline', '--locked', '-p', 'gpui-base', '--release', '--lib',
       '--no-run', '--message-format=json', '-j', '12']
if variant == 'patched':
    cmd += ['--features', 'width-cache-present']
with OUT.joinpath(f'{variant}-build.log').open('w') as stderr:
    result = subprocess.run(cmd, cwd=WORK, stdout=subprocess.PIPE, stderr=stderr, text=True)
OUT.joinpath(f'{variant}-build.jsonl').write_text(result.stdout)
if result.returncode:
    print(OUT.joinpath(f'{variant}-build.log').read_text()[-5000:])
    for line in result.stdout.splitlines():
        event = json.loads(line)
        if event.get('reason') == 'compiler-message':
            print(event['message']['rendered'])
    sys.exit(result.returncode)
for line in result.stdout.splitlines():
    event = json.loads(line)
    if event.get('executable') and event.get('target', {}).get('name') == 'gpui_base':
        binary = OUT / 'bin' / variant
        binary.parent.mkdir(exist_ok=True)
        shutil.copyfile(event['executable'], binary)
        binary.chmod(0o755)
        OUT.joinpath(f'{variant}-build-metadata.json').write_text(json.dumps({
            'variant': variant, 'command': cmd,
            'rustc': subprocess.check_output(['rustc', '-Vv']).decode(),
            'sha256': {name: hashlib.sha256(OUT.joinpath(name).read_bytes()).hexdigest()
                       for name in [f'{variant}-node.rs', 'frame_probe.rs', 'Cargo.lock', f'bin/{variant}']},
        }, indent=2) + '\n')
        print(binary)
