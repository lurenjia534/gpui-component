#!/usr/bin/env python3
"""Run actual old/new paragraph builders in one optimized test binary.

Temporarily adds three unchanged baseline methods under legacy names and an
ignored benchmark test; disables the test-only input counter while timing.
Restores both repository files in finally. No dependencies or manifests change.
"""
from pathlib import Path
import argparse
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('repository', type=Path, help='Checkout of the submitted code commit')
ROOT = parser.parse_args().repository.resolve()
OUT = Path(__file__).resolve().parent
NODE = ROOT / 'crates/base/src/text/node.rs'
INLINE = ROOT / 'crates/base/src/text/inline.rs'


def extract_method(source, name):
    start = source.index('    fn ' + name + '(')
    opening = source.index('{', start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end].replace('fn ' + name + '(', 'fn legacy_' + name + '(', 1)


BENCHMARK = r'''
    fn call(paragraphs: &[Paragraph], mode: usize, legacy: bool, context: &NodeContext, cx: &mut App) {
        for paragraph in paragraphs {
            match (mode, legacy) {
                (0, false) => { std::hint::black_box(paragraph.inline_highlights(context, cx)); }
                (0, true) => { std::hint::black_box(paragraph.legacy_inline_highlights(context, cx)); }
                (1 | 3, false) => { std::hint::black_box(paragraph.plain_render(context, cx)); }
                (1 | 3, true) => { std::hint::black_box(paragraph.legacy_plain_render(context, cx)); }
                (2, false) => { std::hint::black_box(paragraph.inline_flow_items(None, &[], &[], context, cx)); }
                (2, true) => { std::hint::black_box(paragraph.legacy_inline_flow_items(None, &[], &[], context, cx)); }
                _ => unreachable!(),
            }
        }
    }

    fn sample(paragraphs: &[Paragraph], mode: usize, legacy: bool, iterations: usize, context: &NodeContext, cx: &mut App) -> f64 {
        let mut duration = std::time::Duration::ZERO;
        for _ in 0..iterations {
            if mode == 1 {
                for paragraph in paragraphs {
                    *paragraph.render_cache.0.lock().unwrap() = None;
                }
            }
            let start = std::time::Instant::now();
            call(paragraphs, mode, legacy, context, cx);
            duration += start.elapsed();
        }
        duration.as_secs_f64() * 1000. / iterations as f64
    }

    fn check_equivalence(paragraphs: &[Paragraph], context: &NodeContext, cx: &mut App) {
        for paragraph in paragraphs {
            assert_eq!(paragraph.inline_highlights(context, cx), paragraph.legacy_inline_highlights(context, cx));
            *paragraph.render_cache.0.lock().unwrap() = None;
            let old = paragraph.legacy_plain_render(context, cx);
            *paragraph.render_cache.0.lock().unwrap() = None;
            assert_eq!(old, paragraph.plain_render(context, cx));
            let summarize = |items: Vec<InlineFlowItem>| items.into_iter().map(|item| {
                let InlineFlowItem::Text { text, highlights, links, .. } = item else { panic!("expected text") };
                (text, highlights, links)
            }).collect::<Vec<_>>();
            assert_eq!(summarize(paragraph.inline_flow_items(None, &[], &[], context, cx)), summarize(paragraph.legacy_inline_flow_items(None, &[], &[], context, cx)));
        }
    }

    #[test]
    #[ignore = "manual optimized before/after performance measurement"]
    fn paragraph_highlight_performance_comparison() {
        let mut app = gpui::TestApp::new();
        let mut window = app.open_window(|_, _| gpui::Empty);
        window.update(|_, _, cx| {
            cx.set_global(crate::Theme::default());
            let context = NodeContext::default();
            // Assert compatibility for caller-supplied marks spanning later nodes,
            // empty marks, unmarked nodes, and overlapping marks over UTF-8.
            for children in [
                vec![InlineNode::new("a").marks(vec![(0..2, TextMark::default().bold())]), InlineNode::new("bc").marks(vec![(0..2, TextMark::default().italic())])],
                vec![InlineNode::new("a中😀").marks(vec![(0..8, TextMark::default().bold()), (1..8, TextMark::default().italic()), (4..8, TextMark::default().code()), (4..4, TextMark::default().bold())]), InlineNode::new("文"), InlineNode::new("b").marks(vec![(0..1, TextMark::default().italic())])],
            ] {
                check_equivalence(&[Paragraph { children, ..Default::default() }], &context, cx);
            }
            println!("BENCH_HEADER,chunks,paragraphs,nodes,marks,path,before_ms,after_ms,speedup,before_min_ms,before_max_ms,after_min_ms,after_max_ms,before_iters,after_iters");
            for (chunks, paragraph_every) in [(250, 0), (500, 0), (1000, 0), (2000, 0), (500, 20)] {
                let mut source = String::new();
                for i in 0..chunks {
                    source.push_str("Some prose with `inline code` and **bold text** here. ");
                    if paragraph_every > 0 && i % paragraph_every == paragraph_every - 1 { source.push_str("\n\n"); }
                }
                let mut parse_context = NodeContext::default();
                let document = crate::text::format::markdown::parse(&source, &mut parse_context).unwrap();
                let paragraphs = document.blocks.iter().map(|block| {
                    let BlockNode::Paragraph(paragraph) = block else { panic!("expected paragraph") };
                    paragraph.clone()
                }).collect::<Vec<_>>();
                let nodes: usize = paragraphs.iter().map(|p| p.children.len()).sum();
                let marks: usize = paragraphs.iter().flat_map(|p| &p.children).map(|n| n.marks.len()).sum();
                check_equivalence(&paragraphs, &context, cx);
                for (mode, path) in [(0, "inline_highlights"), (1, "plain_render_cold"), (2, "inline_flow_items"), (3, "plain_render_cached")] {
                    // Independently calibrate each implementation to ~15ms per
                    // sample, then alternate measurement order over nine samples.
                    let old_warmup = sample(&paragraphs, mode, true, 1, &context, cx);
                    let new_warmup = sample(&paragraphs, mode, false, 1, &context, cx);
                    let old_iterations = (15. / old_warmup).ceil().clamp(1., 2000.) as usize;
                    let new_iterations = (15. / new_warmup).ceil().clamp(1., 2000.) as usize;
                    let mut old = Vec::new();
                    let mut new = Vec::new();
                    for round in 0..9 {
                        for legacy in if round % 2 == 0 { [true, false] } else { [false, true] } {
                            let value = sample(&paragraphs, mode, legacy, if legacy { old_iterations } else { new_iterations }, &context, cx);
                            if legacy { old.push(value); } else { new.push(value); }
                        }
                    }
                    old.sort_by(f64::total_cmp);
                    new.sort_by(f64::total_cmp);
                    println!("BENCH,{chunks},{},{nodes},{marks},{path},{:.6},{:.6},{:.2},{:.6},{:.6},{:.6},{:.6},{old_iterations},{new_iterations}", paragraphs.len(), old[4], new[4], old[4] / new[4], old[0], old[8], new[0], new[8]);
                }
            }
        });
    }
'''


def main():
    baseline = (OUT / 'before-node.rs').read_text()
    methods = '\n\n'.join(extract_method(baseline, name) for name in ['plain_render', 'inline_highlights', 'inline_flow_items'])
    module = '\n#[cfg(test)]\nmod issue_3695_benchmark {\n    use super::*;\n    impl Paragraph {\n' + methods + '\n    }\n' + BENCHMARK + '\n}\n'
    (OUT / 'benchmark-module.rs').write_text(module)
    final_node = NODE.read_bytes()
    final_inline = INLINE.read_bytes()
    try:
        NODE.write_bytes(final_node + module.encode())
        counter = b'        #[cfg(test)]\n        COMBINED_HIGHLIGHT_INPUTS.with(|inputs| inputs.set(inputs.get() + 1));\n'
        assert final_inline.count(counter) == 1
        INLINE.write_bytes(final_inline.replace(counter, b''))
        command = ['cargo', 'test', '-p', 'gpui-base', '--features', 'test-support', '--lib', '--locked', '--config', 'profile.dev.package.gpui-base.opt-level=3', 'paragraph_highlight_performance_comparison', '--', '--ignored', '--nocapture', '--test-threads=1']
        (OUT / 'benchmark-command.txt').write_text(' '.join(command) + '\n')
        with (OUT / 'benchmark.log').open('w') as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        print((OUT / 'benchmark.log').read_text())
        if result.returncode:
            raise SystemExit(result.returncode)
    finally:
        NODE.write_bytes(final_node)
        INLINE.write_bytes(final_inline)


if __name__ == '__main__':
    main()
