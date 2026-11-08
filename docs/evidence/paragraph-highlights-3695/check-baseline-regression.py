#!/usr/bin/env python3
"""Prove the cost regression test fails on the old four call sites."""
from pathlib import Path
import argparse
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('repository', type=Path, help='Checkout of the submitted code commit')
root = parser.parse_args().repository.resolve()
out = Path(__file__).resolve().parent
node = root / 'crates/base/src/text/node.rs'
final = node.read_bytes()
current = b'append_node_highlights(&mut highlights, node_highlights);'
original = b'highlights = combine_highlights(highlights, node_highlights);'
assert final.count(current) == 4
assert b'issue_3695_benchmark' not in final
try:
    node.write_bytes(final.replace(current, original))
    command = ['cargo', 'test', '-p', 'gpui-base', '--features', 'test-support', '--lib', '--locked', 'paragraph_highlight', '--', '--test-threads=1']
    with (out / 'baseline-regression.log').open('w') as log:
        result = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
    output = (out / 'baseline-regression.log').read_text()
    print(output)
    assert result.returncode == 101
    assert 'paragraph_highlight_builders_merge_each_mark_once ... FAILED' in output
    assert 'paragraph_highlights_preserve_overlaps_and_utf8_boundaries ... ok' in output
    assert 'left: 6240' in output and 'right: 192' in output
finally:
    node.write_bytes(final)
