use super::*;
use gpui::{AppContext as _, TestAppContext};
use crate::text::{TextViewState, format::markdown, node::NodeContext};
use std::{hint::black_box, time::Instant};

#[gpui::test]
#[ignore]
fn validation_table_highlight_remap_scaling(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let phase = std::env::var("HIGHLIGHT_PHASE").unwrap_or_else(|_| "unknown".into());
    let shapes = [(1000, 1), (2000, 1), (4000, 1), (8000, 1),
        (16000, 1), (32000, 1), (64000, 1), (100000, 1), (2, 4096), (64, 64)];
    println!("phase,rows,columns,highlighted_cells,source_bytes,build_ms,remap_ms,total_ms,kept_cells");
    for (rows, columns) in shapes {
        let row = format!("|{}\n", " x |".repeat(columns));
        let separator = format!("|{}\n", "---|".repeat(columns));
        let source = format!("{row}{separator}{}", row.repeat(rows));
        let view = cx.update(|cx| cx.new(|cx| TextViewState::markdown(&source, cx)));
        cx.run_until_parked();
        let text = view.read_with(cx, |state, _| state.rendered_text());
        let highlights: Vec<_> = text.as_str().match_indices('x')
            .map(|(start, _)| RangeHighlight::new(start..start + 1, gpui::hsla(0.15, 1., 0.5, 0.4)))
            .collect();
        let count = (rows + 1) * columns;
        assert_eq!(highlights.len(), count);
        // Exercise the public setter too; the same resolved frame is timed below.
        view.update(cx, |state, cx| state.set_range_highlights(highlights.clone(), cx).unwrap());
        let frame = RangeHighlightFrame::new(&text, highlights).unwrap().unwrap();
        assert_eq!(frame.leaves.len(), count);
        let old = &text.document;
        let edit = row.len() + separator.len() + row.rfind('x').unwrap();
        let mut changed = source.clone();
        changed.replace_range(edit..edit + 1, "y");
        let new = markdown::parse(&changed, &mut NodeContext::default()).unwrap();
        let mut samples = Vec::new();
        // Warm-up plus five timed samples; no parsing/search/setup in the timer.
        for repetition in 0..6 {
            let start = Instant::now();
            let remap = LeafRemap::new(black_box(old), black_box(&new), false);
            let build_ms = start.elapsed().as_secs_f64() * 1000.;
            let start = Instant::now();
            let kept = black_box(frame.remap(black_box(&remap))).unwrap();
            let remap_ms = start.elapsed().as_secs_f64() * 1000.;
            assert_eq!(kept.leaves.len(), columns);
            for column in 0..columns {
                assert_eq!(kept.backgrounds(TextLeafKey::table_cell(0, column)),
                    &[(0..1, gpui::hsla(0.15, 1., 0.5, 0.4))]);
            }
            if repetition > 0 { samples.push((build_ms + remap_ms, build_ms, remap_ms)); }
        }
        samples.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (total, build, remap) = samples[2];
        println!("{phase},{rows},{columns},{count},{},{build:.6},{remap:.6},{total:.6},{columns}", source.len());
        drop(view);
        cx.run_until_parked();
    }
}
