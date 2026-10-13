#!/usr/bin/env python3
"""Recreate #3468 evidence using exact source revisions and real GPUI rendering."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


BASE = "8d4907da456a4f363c7ee1c22c553f44c3c58500"
FIX = "0825fb2f5ec5f9c1a2c93ef01cc57961d5c4443a"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="slider-3468-output")
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    evidence = Path(__file__).resolve().parent
    repo = evidence.parents[1]
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix="gpui-slider-3468-reproduce-"))
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(repo / "target"))
    env["CARGO_TERM_COLOR"] = "never"
    offline = ["--offline"] if args.offline else []

    def run(command, log, expected_exit):
        result = subprocess.run(
            command, env=env, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True,
        )
        (output / log).write_text(result.stdout)
        print(f"{log}: exit {result.returncode}", flush=True)
        if result.returncode != expected_exit:
            raise RuntimeError(result.stdout[-6000:])
        return result.stdout

    for phase, revision in [("before", BASE), ("after", FIX)]:
        tree = scratch / phase
        tree.mkdir()
        archive = scratch / f"{phase}.tar"
        with archive.open("wb") as stream:
            subprocess.run(
                ["git", "-C", str(repo), "archive", revision],
                stdout=stream, check=True,
            )
        with tarfile.open(archive) as stream:
            stream.extractall(tree, filter="data")
        fixture = scratch / f"fixture-{phase}"
        shutil.copytree(evidence / "fixture", fixture)
        manifest = fixture / "Cargo.toml"
        manifest.write_text(manifest.read_text().replace(
            "../../../crates/kit", str(tree / "crates/kit"),
        ))
        api_log = run(
            ["cargo", "check", "--locked", *offline, "--features", "fixed",
             "--manifest-path", str(manifest)],
            f"{phase}-api-check.log", 101 if phase == "before" else 0,
        )
        if phase == "before":
            assert api_log.count("error[E0599]") == 2, api_log
        features = ["--features", "fixed"] if phase == "after" else []
        run(
            ["cargo", "run", "--locked", *offline, *features,
             "--manifest-path", str(manifest), "--", str(output), phase],
            f"{phase}-render.log", 1 if phase == "before" else 0,
        )

    before = json.loads((output / "before.json").read_text())
    after = json.loads((output / "after.json").read_text())
    assert not before["all_name_contracts_passed"]
    assert after["all_name_contracts_passed"]
    assert all(
        node["label"] is None
        for theme in before["results"]
        for case in theme["cases"]
        for node in case["nodes"]
    )
    for before_theme, after_theme in zip(before["results"], after["results"], strict=True):
        for before_case, after_case in zip(before_theme["cases"], after_theme["cases"], strict=True):
            for before_node, after_node in zip(before_case["nodes"], after_case["nodes"], strict=True):
                assert {k: v for k, v in before_node.items() if k != "label"} == {
                    k: v for k, v in after_node.items() if k != "label"
                }
    for theme in ["light", "dark"]:
        assert (output / f"before-{theme}-controls.png").read_bytes() == (
            output / f"after-{theme}-controls.png"
        ).read_bytes()
    (output / "reproduction-summary.json").write_text(json.dumps({
        "baseline": BASE,
        "fix": FIX,
        "snapshots": str(scratch),
        "all_after_name_contracts_passed": True,
        "controls_png_byte_identical": True,
        "png_sha256": {
            p.name: hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(output.glob("*.png"))
        },
    }, indent=2) + "\n")
    print(f"Evidence reproduced. Output: {output}; snapshots: {scratch}", flush=True)


if __name__ == "__main__":
    main()
