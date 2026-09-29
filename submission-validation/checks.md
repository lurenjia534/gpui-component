# Validation after synchronizing upstream

Code commit: `de586ce32f2e8bc6d3047897bf234ba66584544d`.
Parent: `4f861e8a0654064fa53255cf8d9597bb9d5ef826`.

| Command | Result |
| --- | --- |
| `cargo test --offline --locked -p gpui-base --lib table_column -- --test-threads=1` | 5 passed, 0 failed |
| `cargo test --offline --locked -p gpui-base --lib` | 1,247 passed, 0 failed; complete output in `base-tests.log` |
| `cargo fmt -p gpui-base -- --check` | Passed |
| `cargo clippy --offline --locked -p gpui-base --lib --tests -- -D warnings` | Passed; output in `base-clippy.log` |
| `git diff --check` | Passed before the code commit |

Performance measurements remain on the earlier frozen baseline. They were not
rerun after synchronization. The code source and lockfile are unchanged between
the measurement and submission contexts; see `../PROVENANCE.json`.

The complete workspace, Story Gallery and macOS/Windows were not rerun here.
The first full-test tool response was truncated and is retained as
`base-tests-tool-excerpt.log`. The same suite was immediately rerun with direct
file capture; `base-tests.log` contains the complete passing output.
`Clippy --all-targets` is not claimed: the previous measurement report records an
unrelated Criterion 0.5/0.8 benchmark dependency conflict. Dynamic-font installation
invalidation remains unresolved, so the PR is a draft.
