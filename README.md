# Table highlight remapping verification

Baseline: `f8cd486005ab1c09fd6cf9fc00e8a4ca42a4ad17`.
Fix: `27fbe408adac8cf6fc14045ac4ed1c653f3ada9a`.

These are supporting artifacts for a one-source-file fix. The screenshots show actual native-terminal Rust benchmark results, not GUI frame-latency measurements.

## Method

Linux x86_64, AMD Ryzen 9 7950X, Rust/Cargo 1.98.0. Only gpui-base is overridden to opt-level=3; dependencies use the repository's dev profile. Each shape uses one warm-up and five timed samples. The reported sample is the median by build + remap time. Parsing, rendered-text search, and highlight installation are excluded; LeafRemap construction, including the new row index, is included.

The benchmark uses the production Markdown parser, rendered-text index, public set_range_highlights setter and RangeHighlightFrame::remap. Every x is highlighted. The final cell of the first data row is changed to y. It asserts that header highlights remain and all cells in the edited and following rows lose their highlights.

For 100,001 cells (600,012 source bytes), total reconciliation fell from 1,467.864 ms to 10.104 ms. A 3-row / 4,096-column table fell from 180.165 ms to 0.866 ms. These are measured cases, not a general frame-time guarantee.

## Reproduce the benchmark

Use separate disposable checkouts of the baseline and fix commits. In each checkout, copy `benchmark.rs` from this evidence branch to `target/table-highlight-verification/benchmark.rs`, and append the following temporary child module to `crates/base/src/text/range_highlight.rs`:

```rust
#[cfg(test)]
mod table_highlight_validation {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/table-highlight-verification/benchmark.rs"));
}
```

Run this in the baseline checkout and change `before` to `after` in the fix checkout:

```sh
HIGHLIGHT_PHASE=before cargo --config 'profile.dev.package.gpui-base.opt-level=3' test --locked -p gpui-base --lib validation_table_highlight_remap_scaling -- --ignored --nocapture
```

This temporary module is not included in the fix commit.

## Tests

The final fix commit's source passed `cargo test --locked -p gpui-base --lib`: 1,152 passed, 0 failed. `cargo fmt --all -- --check`, `git diff --check`, and `cargo clippy --locked -p gpui-base --all-targets -- -D warnings` also passed.

A separate temporary `end_to_end.rs` child module in `text/state.rs` exercised a 2,002-highlight table above the background-parse threshold through public set_text. It verified the source commit, preserved header highlights and reveal, and cleared the edited and following rows. It passed and was removed from production source after validation.

The Markdown example built and displayed on Linux. This was a startup smoke check; neither full interactive GUI stress testing nor macOS/Windows performance testing was completed.
