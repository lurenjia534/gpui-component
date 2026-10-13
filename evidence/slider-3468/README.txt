Slider #3468 verification artifacts

Production commit: 0825fb2f5ec5f9c1a2c93ef01cc57961d5c4443a
Clean upstream baseline: 8d4907da456a4f363c7ee1c22c553f44c3c58500

PNG files are real Linux GPUI WGPU offscreen captures, unedited.
The lower inspector is a review-only view reading actual GPUI node labels.
State values are read from SliderState. Element identity paths refer to GPUI
rendered elements; they are not OS accessibility node IDs.

before.json and after.json contain rendered node observations for both themes
across English naming, Chinese renaming and removal. validation-summary.json
contains the comparison results and PNG/patch SHA-256 hashes.

The before build exposes no accessibility_label method on either layer (two
E0599 errors) and has empty node names. The after build passes all naming cases.
Controls-only screenshots are byte-identical, hence pixel-identical, in both
themes. Node roles, element identity paths, state values and bounds match.

To run the after fixture from this evidence branch:
mkdir -p review-output
cargo run --locked --manifest-path evidence/slider-3468/fixture/Cargo.toml --features fixed -- review-output after

To recreate both clean builds, run:
python3 evidence/slider-3468/reproduce.py --output review-output

The reproduction script uses git archive of the exact two commits and checks
expected compilation/runtime exit codes. Its temporary snapshots are retained
for inspection. It requires Cargo, cached or downloadable dependencies, and a
working native GPUI headless renderer. It uses real rendering throughout.

checks/ contains the product tests, check, Clippy, formatting and diff-check
logs. Ten relevant tests passed. Use online Cargo if dependencies are missing.

Native screen-reader speech, the native Story window and macOS/Windows runtime
verification have not been performed. Evidence does not imply those checks.
