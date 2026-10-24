# Sparse Markdown table allocation: before and after

[中文](README.zh-CN.md)

Validated on 2026-10-24 against base commit
`69a2077c2da588de1c46b89ffd903e1373e45a29`, with
`gpui-kit-sparse-table.patch` applied on `fix/sparse-markdown-table`.
Code commit: `0e7a58062a94dcbea667d6f2962140fe1d8538de`.
Toolchain: rustc 1.98.0, cargo 1.98.0, Linux. This report and validation
tooling were prepared with AI assistance.

## Result

The production Markdown converter retains only source-provided cells after
the fix. Before the fix, it pads every body row to the header width, retaining
quadratically many cells for compact sparse input. These are **measured
production `TableCell` counts**, not dependency-only AST counts or estimates.

| Header columns | Body rows | Input bytes | Before: retained cells | After: retained cells | Eliminated generated cells |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 64 | 452 | 2,080 | 96 | 1,984 |
| 64 | 128 | 900 | 8,256 | 192 | 8,064 |
| 128 | 256 | 1,796 | 32,896 | 384 | 32,512 |

Raw output: [before.log](before.log), [after.log](after.log).
Doubling both dimensions approximately quadruples retained cells before the
fix, while it doubles them after the fix.

## Method and reproduction

The temporary [probe.rs](probe.rs) calls the actual
`text::format::markdown::parse` function, then sums the retained
`TableRow.children.len()` values in the resulting document. It uses:

```rust
format!(
    "{}|\n{}|\n{}",
    "|h".repeat(columns),
    "|---".repeat(columns),
    "|x|\n".repeat(rows),
)
```

The before run restores the original `resize_with` sink in the patched
source; all other production code is identical. The after run uses `truncate`.
The same bounded probe runs in both versions and never attempts an OOM.
The probe was removed from production source after validation; only the
patch's regression tests remain there.

To reproduce from the patched repository, with no concurrent source edits:

```sh
python3 docs/evidence/sparse-markdown-table/reproduce.py
```

The script temporarily installs the probe and restores the source in a
`finally` block. Each Cargo invocation uses:

```sh
cargo test -p gpui-base --features test-support --offline --lib \
  text::format::markdown::sparse_table_evidence -- --exact --nocapture
```

The `test-support` feature is required by existing tests in this checkout.
An initial invocation without it failed to compile because
`a11y_disabled.rs` references feature-gated `crate::test_support`; enabling
the feature resolved that failure. Offline dependencies were available in
this environment.

## Regression checks

- The new sparse-cell regression **fails with the original padding**:
  [before-regression.log](before-regression.log), exit 101. It fails the
  assertion that each one-cell body row retains one cell.
- The same regression **passes with the fix**:
  [after-regression.log](after-regression.log), exit 0.
- Ragged-row semantics **pass with the fix**:
  [ragged-regression.log](ragged-regression.log). The test checks a short
  row, truncation of excess cells, and preservation of an explicitly empty
  middle cell. Resulting row lengths are `[3, 1, 3, 3]`.
- `cargo fmt --all --check` and `git diff --check` passed.
  [format.log](format.log) is empty because the format check succeeded.

Base library/tests Clippy also passed:

```sh
cargo clippy --locked --offline -p gpui-base --features test-support --lib --tests -- -D warnings
```

Raw output: [clippy.log](clippy.log).

## Why the change addresses the finding

Before:

```rust
row.children.resize_with(table.column_aligns.len(), Default::default);
table.children.push(row);
```

After:

```rust
row.children.truncate(table.column_aligns.len());
table.children.push(row);
```

For this input shape, input size is `6*C + 4*R + 4` bytes. The old converter
retains `C*(R+1)` cells, including `R*(C-1)` synthesized cells; the fix retains
`C+R`. A default `TableCell` owns a default `Paragraph`, which contains
`Arc<Mutex<InlineState>>`, so padding creates nontrivial retained objects.

For `C=R=10,000`, the formulas predict 100,010,000 retained cells before and
20,000 after, from 100,004 input bytes. **This large case was not executed in
this validation.** It is an extrapolation from the code and bounded results,
not a measured allocation, RSS, or crash result.

## Reachability and alignment review

Current source references:

- `crates/base/src/text/text_view.rs`: `TextView::markdown` accepts text;
  layout constructs `TextViewState::markdown`.
- `crates/base/src/text/state.rs`: `parse_content` calls
  `format::markdown::parse` for Markdown, including reparsed appended content.
- `crates/base/src/text/format/markdown.rs`: `Node::Table` derives
  `column_aligns` from the parser's alignment list; each row reaches
  `parse_table_row`.
- `crates/base/src/text/node.rs`: `TableCell` owns `Paragraph`.
  `BlockNode::render_table_row` iterates the shared columns and uses
  `row.children.get(ix)` for optional content. Missing trailing cells therefore
  do not require stored placeholder paragraphs to occupy their column slots.
- `crates/component/src/input/popovers/hover_popover.rs` forwards hover
  documentation through `render_markdown` in `popovers/mod.rs` to
  `TextView::markdown`. A configured provider and user interaction are needed
  for this route; providers default to absent.

An application must render attacker-influenced Markdown for this to cross a
trust boundary. A public renderer API does not establish public network
exposure. The synchronous/background parsing threshold is scheduling, not
an allocation budget.

## Limits

This evidence verifies the introduced **retained parser-cell expansion**.
No GUI screenshots, visual alignment run, end-to-end hover interaction,
peak-RSS measurement, deliberate OOM, or full test suite run is claimed.
Alignment compatibility was reviewed in renderer source and parser regression
tests, not established by a screenshot.

The fix does not impose a general document budget or remove upstream parser
CPU amplification. The renderer still visits the shared column grid for
each row and creates presentation elements; this report does not establish
bounded layout/rendering cost for hostile tables.

## Suggested PR description

Title: `text: Preserve sparse Markdown table rows`

Preserve missing Markdown table cells implicitly instead of allocating a
default `TableCell` for every absent header column. Continue truncating excess
cells to the GFM header width; the renderer already handles missing content
on the shared column grid.

Bounded production-parser validation reduces retained cells from 32,896 to
384 for a 1,796-byte table. The sparse-cell regression fails with the old
padding and passes with the fix; ragged-row semantics and formatting pass.
No public API changes. AI-assisted patch validation and evidence preparation;
human review remains required.
