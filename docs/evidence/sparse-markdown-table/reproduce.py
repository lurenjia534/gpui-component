"""Run bounded production-parser comparisons; always restore the source file.

Run from any directory: python3 docs/evidence/sparse-markdown-table/reproduce.py
Requires the applied sparse-table patch and its two regression tests.
AI-assisted validation tooling; review before use.
"""

from pathlib import Path
import subprocess


evidence = Path(__file__).resolve().parent
root = evidence.parents[2]
source = root / "crates/base/src/text/format/markdown.rs"
fixed = source.read_text()
sparse = "row.children.truncate(table.column_aligns.len());"
dense = "row.children.resize_with(table.column_aligns.len(), Default::default);"
assert fixed.count(sparse) == 1, "Apply the sparse-table patch first"
assert "fn sparse_table_evidence()" not in fixed
probe = (evidence / "probe.rs").read_text()
command = [
    "cargo", "test", "-p", "gpui-base", "--features", "test-support",
    "--offline", "--lib",
]


def run(name, test, expected=0):
    with (evidence / name).open("w") as log:
        result = subprocess.run(
            command + [test, "--", "--exact", "--nocapture"],
            cwd=root, stdout=log, stderr=subprocess.STDOUT,
        )
    assert result.returncode == expected, f"See {name}: exit {result.returncode}"


try:
    source.write_text(fixed.replace(sparse, dense) + probe)
    run("before.log", "text::format::markdown::sparse_table_evidence")
    run(
        "before-regression.log",
        "text::format::markdown::tests::sparse_table_cell_count_tracks_source_cells",
        expected=101,
    )
    source.write_text(fixed + probe)
    run("after.log", "text::format::markdown::sparse_table_evidence")
    source.write_text(fixed)
    run(
        "after-regression.log",
        "text::format::markdown::tests::sparse_table_cell_count_tracks_source_cells",
    )
    run(
        "ragged-regression.log",
        "text::format::markdown::tests::table_rows_keep_missing_cells_implicit",
    )
finally:
    source.write_text(fixed)

print("Production parser comparisons complete; source restored.")
