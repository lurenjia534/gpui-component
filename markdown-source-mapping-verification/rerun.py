#!/usr/bin/env python3
"""Reinstall the temporary Rust validation module, run it, and restore the file."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile

arguments = argparse.ArgumentParser()
arguments.add_argument("--repo", type=Path, default=Path.cwd())
arguments.add_argument("--benchmark", action="store_true")
options = arguments.parse_args()
folder = Path(__file__).resolve().parent
repo = options.repo.resolve()
production = repo / "crates/base/src/text/format/markdown.rs"
original = production.read_bytes()
if original != (folder / "markdown-after.rs").read_bytes():
    raise SystemExit("Production source differs from the verified patched snapshot; stopped.")
baseline = (folder / "markdown-before.rs").read_text()
if (folder / "markdown-before-core.rs").read_text() != baseline.split("#[cfg(test)]\nmod tests {")[0]:
    raise SystemExit("The archived baseline parser differs from its source snapshot; stopped.")
output_parent = repo / "target/markdown-source-mapping-reproduction"
output_parent.mkdir(parents=True, exist_ok=True)
output = Path(tempfile.mkdtemp(prefix="run-", dir=output_parent))
log = output / "validation.log"
module = (
    '\n#[cfg(test)]\nmod source_mapping_validation {\n'
    f'    include!("{folder / "benchmark.rs"}");\n'
    '}\n'
).encode()
installed = original + module
command = [
    "cargo", "--config", "profile.dev.package.gpui-base.opt-level=3",
    "test", "--locked", "-p", "gpui-base", "--lib",
]
if options.benchmark:
    command.extend([
        "validation_markdown_source_mapping_scaling", "--", "--ignored",
        "--nocapture", "--test-threads=1",
    ])
else:
    command.extend(["validation_markdown_", "--", "--nocapture", "--test-threads=1"])
environment = os.environ.copy()
environment["MARKDOWN_VERIFICATION_DIR"] = str(output)
try:
    production.write_bytes(installed)
    print("Running:", " ".join(command), flush=True)
    print("New results:", output, flush=True)
    with log.open("w") as transcript:
        process = subprocess.Popen(
            command, cwd=repo, env=environment, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True,
        )
        for line in process.stdout:
            print(line, end="", flush=True)
            transcript.write(line)
        status = process.wait()
finally:
    if production.read_bytes() == installed:
        production.write_bytes(original)
    else:
        raise RuntimeError("Production source changed during validation; not overwritten.")
raise SystemExit(status)
