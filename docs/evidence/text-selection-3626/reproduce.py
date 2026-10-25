#!/usr/bin/env python3
"""Rebuild the bounded issue 3626 regression and performance comparison."""
import argparse
from pathlib import Path
import shutil
import subprocess

BASE = '7b14e03f2ed4b46fc4149a8a523c7f91bcad6560'
FIX = '9b42fb299a50610d83c5fb484cf4db390871ac28'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True, help='New directory for worktrees, logs, captures and build outputs')
parser.add_argument('--measure', action='store_true', help='Also run real-font/GPU performance fixture and capture screenshots')
args = parser.parse_args()
repo = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip())
evidence = Path(__file__).resolve().parent
out = args.output.resolve()
if out.exists():
    raise SystemExit('Choose a new output directory; existing outputs are not overwritten.')
out.mkdir(parents=True)
for rev in (BASE, FIX):
    subprocess.run(['git', '-C', str(repo), 'cat-file', '-e', rev+'^{commit}'], check=True)
fixed_source = subprocess.check_output(['git', '-C', str(repo), 'show', FIX+':crates/base/src/text_selection.rs'], text=True)
helpers = fixed_source[fixed_source.index('    struct RetainedSelectionText {'):fixed_source.index('    #[gpui::test]\n    fn active_selection_frame_publication_visits_participants_linearly')]
regression = 'text_selection::tests::active_selection_keeps_unrelated_text_views_retained'
for label, rev in [('before', BASE), ('after', FIX)]:
    work = out / label
    subprocess.run(['git', '-C', str(repo), 'worktree', 'add', '--detach', str(work), rev], check=True)
    if label == 'before':
        path = work/'crates/base/src/text_selection.rs'
        source = path.read_text()
        marker = "    /// A view reading the window's selection state, beside selectable text,"
        source = source.replace(marker, helpers+marker, 1)
        path.write_text(source)
    import os
    env = dict(os.environ, CARGO_TARGET_DIR=str(out/('target-'+label)))
    test = ['cargo', 'test', '--locked', '-p', 'gpui-base', '--features', 'gpui-fast,test-support', '--lib', regression, '--', '--exact', '--nocapture']
    result = subprocess.run(test, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (out/(label+'-regression.log')).write_text(result.stdout)
    if label == 'before':
        assert result.returncode == 101 and 'active registration redrew an unrelated view' in result.stdout, result.stdout
    else:
        assert result.returncode == 0 and '1 passed' in result.stdout, result.stdout
    if args.measure:
        fixture = work/'crates/kit/examples/selection_3626.rs'
        fixture.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(evidence/'performance/selection_3626.rs', fixture)
        command = ['cargo', 'run', '--locked', '-p', 'gpui-kit', '--example', 'selection_3626', '--features', 'gpui-fast,test-support', '--', label, str(out)]
        with (out/(label+'.csv')).open('w') as samples, (out/(label+'-run.log')).open('w') as log:
            subprocess.run(command, cwd=work, env=env, stdout=samples, stderr=log, check=True)
print('PASS: expected baseline regression failure and fixed regression success.')
print('Evidence output:', out)
print('Worktrees are retained for inspection. Each version uses its own Cargo target directory.')
