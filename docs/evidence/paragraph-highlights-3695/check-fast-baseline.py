#!/usr/bin/env python3
"""Compare gpui-fast test failures against the exact original two files."""
from pathlib import Path
import argparse
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('repository', type=Path, help='Checkout of the submitted code commit')
root = parser.parse_args().repository.resolve()
out = Path(__file__).resolve().parent
paths = [root / 'crates/base/src/text' / name for name in ['node.rs', 'inline.rs']]
final = {path: path.read_bytes() for path in paths}
for path in paths:
    assert final[path] == (out / ('after-' + path.name)).read_bytes()
try:
    for path in paths:
        path.write_bytes((out / ('before-' + path.name)).read_bytes())
    command = ['cargo', 'test', '-p', 'gpui-base', '--features', 'gpui-fast,test-support', '--lib', '--locked', 'text::', '--', '--test-threads=1']
    with (out / 'text-tests-fast-baseline.log').open('w') as log:
        result = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
    baseline = (out / 'text-tests-fast-baseline.log').read_text()
    patched = (out / 'text-tests-fast.log').read_text()
    failures = lambda text: set(re.findall(r'^test (\S+) \.\.\. FAILED$', text, re.MULTILINE))
    print('Baseline exit code:', result.returncode)
    print('Baseline failing tests:', sorted(failures(baseline)))
    print('Patched failing tests:', sorted(failures(patched)))
    print('\n'.join(baseline.splitlines()[-12:]))
    assert result.returncode == 101
    assert len(failures(baseline)) == 5
    assert failures(baseline) == failures(patched)
finally:
    for path, content in final.items():
        path.write_bytes(content)
