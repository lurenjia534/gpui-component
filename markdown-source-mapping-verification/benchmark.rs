// Temporary validation module; never shipped in the production crate.
use super::*;
use crate::text::format::html;
use gpui::AppContext;
use std::{hint::black_box, time::Instant};

#[allow(dead_code)]
mod baseline {
    include!("markdown-before-core.rs");

    pub(super) fn align(
        raw: &str,
        rendered: &str,
        offset: usize,
        decode: bool,
    ) -> Vec<SourceSegment> {
        aligned_source_segments(raw, rendered, offset, decode)
    }
}

fn measure<T>(mut operation: impl FnMut() -> T) -> (u128, Vec<u128>) {
    drop(black_box(operation()));
    let mut samples = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let result = operation();
        samples.push(start.elapsed().as_nanos());
        drop(black_box(result));
    }
    let mut sorted = samples.clone();
    sorted.sort_unstable();
    (sorted[1], samples)
}

#[test]
#[ignore = "bounded before/after CPU benchmark; run explicitly"]
fn validation_markdown_source_mapping_scaling() {
    let output = std::env::var("MARKDOWN_VERIFICATION_DIR").unwrap();
    let output_file = format!("{output}/results.json");
    // Keep finished cases if an explicit benchmark run was interrupted.
    let mut results: Vec<serde_json::Value> = std::fs::read(&output_file)
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
        .unwrap_or_default();
    let cases = [
        (
            "alternating_spaces",
            vec![32768, 65536, 98304, 131072, 262144],
        ),
        ("horizontal_whitespace", vec![8192, 16384, 32768, 65536]),
        (
            "multiline_code",
            vec![8192, 16384, 32768, 65536, 131072, 262144],
        ),
        ("unmapped_characters", vec![8192, 16384, 32768, 65536]),
        ("entity_after_gaps", vec![8192, 16384, 32768, 65536]),
    ];
    for (kind, sizes) in cases {
        for size in sizes {
            let (raw, rendered, offset, decode, source) = match kind {
                "alternating_spaces" => {
                    let raw = "a ".repeat(size / 2);
                    let rendered = raw.trim_end_matches(' ').to_string();
                    let source = Some(raw.clone());
                    (raw, rendered, 0, true, source)
                }
                "horizontal_whitespace" => {
                    let raw = format!("a{}b", " ".repeat(size - 2));
                    let source = Some(raw.clone());
                    (raw.clone(), raw, 0, true, source)
                }
                "multiline_code" => {
                    let raw = "x\n".repeat(size / 2);
                    let rendered = raw[..raw.len() - 1].to_string();
                    let source = Some(format!("```\n{raw}```"));
                    (raw, rendered, 4, false, source)
                }
                "unmapped_characters" => (
                    "abc".repeat(size / 3),
                    format!("{}abc", "\u{fffd}".repeat(size / 3)),
                    7,
                    true,
                    None,
                ),
                "entity_after_gaps" => (
                    format!("&{};", "a".repeat(size - 2)),
                    format!("{}&a", "\u{fffd}".repeat(size / 3)),
                    9,
                    true,
                    None,
                ),
                _ => unreachable!(),
            };
            if results.iter().any(|result| {
                result["kind"] == kind && result["raw_bytes"] == raw.len()
            }) {
                continue;
            }
            // Verify exact mapping semantics independently of the timing.
            let old = baseline::align(&raw, &rendered, offset, decode);
            let new = aligned_source_segments(&raw, &rendered, offset, decode);
            assert_eq!(old, new, "{kind} {size}");
            let old_capacity = old.capacity();
            let new_capacity = new.capacity();
            let segment_count = new.len();
            drop((old, new));
            let (old_ns, old_samples) =
                measure(|| baseline::align(black_box(&raw), black_box(&rendered), offset, decode));
            let (new_ns, new_samples) = measure(|| {
                aligned_source_segments(black_box(&raw), black_box(&rendered), offset, decode)
            });
            let mut result = serde_json::json!({
                "kind": kind,
                "raw_bytes": raw.len(),
                "rendered_bytes": rendered.len(),
                "segment_count": segment_count,
                "segment_size_bytes": std::mem::size_of::<SourceSegment>(),
                "old_capacity": old_capacity,
                "new_capacity": new_capacity,
                "old_align_ns": old_ns,
                "new_align_ns": new_ns,
                "old_align_samples_ns": old_samples,
                "new_align_samples_ns": new_samples,
            });
            if let Some(source) = source {
                let before = baseline::parse(&source, &mut NodeContext::default()).unwrap();
                let after = parse(&source, &mut NodeContext::default()).unwrap();
                assert_eq!(before, after, "parsed document: {kind} {size}");
                drop((before, after));
                let (old_parse, old_samples) = measure(|| {
                    baseline::parse(black_box(&source), &mut NodeContext::default()).unwrap()
                });
                let (new_parse, new_samples) =
                    measure(|| parse(black_box(&source), &mut NodeContext::default()).unwrap());
                result["source_bytes"] = serde_json::json!(source.len());
                result["old_parse_ns"] = serde_json::json!(old_parse);
                result["new_parse_ns"] = serde_json::json!(new_parse);
                result["old_parse_samples_ns"] = serde_json::json!(old_samples);
                result["new_parse_samples_ns"] = serde_json::json!(new_samples);
            }
            println!("RESULT {result}");
            results.push(result);
            let output = std::env::var("MARKDOWN_VERIFICATION_DIR").unwrap();
            std::fs::write(
                format!("{output}/results.json"),
                serde_json::to_vec_pretty(&results).unwrap(),
            )
            .unwrap();
        }
    }
    assert_eq!(results.len(), 23);
}

fn words(alphabet: &[&str], depth: usize) -> Vec<String> {
    let mut output = vec![String::new()];
    let mut level = vec![String::new()];
    for _ in 0..depth {
        level = level
            .iter()
            .flat_map(|prefix| {
                alphabet
                    .iter()
                    .map(move |suffix| format!("{prefix}{suffix}"))
            })
            .collect();
        output.extend(level.iter().cloned());
    }
    output
}

#[test]
fn validation_markdown_alignment_rust_differential() {
    let values = words(&["a", "b", " ", "\t", "\n", "\r", "\\", "&", "中"], 3);
    let mut count = 0;
    for raw in &values {
        for rendered in &values {
            for decode in [false, true] {
                assert_eq!(
                    baseline::align(raw, rendered, 7, decode),
                    aligned_source_segments(raw, rendered, 7, decode),
                    "raw={raw:?}, rendered={rendered:?}, decode={decode}"
                );
                count += 1;
            }
        }
    }
    let entities = [
        "&amp;",
        "&#65;",
        "&#x1F600;",
        "&NotEqualTilde;",
        "&bad;",
        "&a",
        "&#0;",
        "&#xD800;",
        "&",
    ];
    let decorations = ["", "a", " ", "\t", "\r\n", "\\", "\u{fffd}", "中"];
    let rendered_values = words(
        &[
            "a",
            "&",
            "\n",
            " ",
            "\u{fffd}",
            "中",
            "😀",
            "\u{2242}\u{338}",
        ],
        3,
    );
    for entity in entities {
        for prefix in decorations {
            for suffix in decorations {
                let raw = format!("{prefix}{entity}{suffix}");
                for rendered in &rendered_values {
                    for decode in [false, true] {
                        assert_eq!(
                            baseline::align(&raw, rendered, 19, decode),
                            aligned_source_segments(&raw, rendered, 19, decode),
                            "raw={raw:?}, rendered={rendered:?}, decode={decode}"
                        );
                        count += 1;
                    }
                }
            }
        }
    }
    println!("DIFFERENTIAL cases={count}");
}

#[gpui::test]
fn validation_markdown_public_background_path(cx: &mut gpui::TestAppContext) {
    use crate::TextViewState;
    cx.update(crate::init);
    let paragraph = "a ".repeat(49_152);
    let body = "x\n".repeat(49_152);
    for source in [paragraph.clone(), format!("```\n{body}```")] {
        let state = cx.update(|cx| cx.new(|cx| TextViewState::markdown(&source, cx)));
        // Large initial inputs must complete through the background parser.
        assert!(source.len() > 4096);
        cx.run_until_parked();
        state.read_with(cx, |state, _| {
            assert_eq!(state.source().as_str(), source);
            assert!(!state.has_view_selection());
            let rendered = state.rendered_text();
            let expected = if source == paragraph {
                "a ".repeat(49_152).trim_end().to_string()
            } else {
                body.trim_end().to_string()
            };
            assert_eq!(rendered.as_str().trim_end(), expected);
        });
        state.update(cx, |state, cx| {
            state.set_text(&format!("{paragraph}end"), cx)
        });
        cx.run_until_parked();
        state.read_with(cx, |state, _| {
            assert_eq!(state.source().as_str(), format!("{paragraph}end"));
            assert_eq!(
                state.rendered_text().as_str().trim_end(),
                format!("{paragraph}end")
            );
            assert!(!state.has_view_selection());
        });
    }
}
