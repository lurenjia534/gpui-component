use super::*;
use gpui::TestAppContext;
use crate::text::stream_fade::TextLeafKey;

#[gpui::test]
fn validation_large_table_set_text_background_commit(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let source = format!("| x | x |\n|---|---|\n{}", "| x | x |\n".repeat(1000));
    assert!(source.len() > MAX_SYNC_FULL_REPLACE_BYTES);
    let view = cx.update(|cx| cx.new(|cx| TextViewState::markdown(&source, cx)));
    cx.run_until_parked();
    view.update(cx, |state, cx| {
        let text = state.rendered_text();
        let highlights: Vec<_> = text.as_str().match_indices('x')
            .map(|(start, _)| RangeHighlight::new(start..start + 1, gpui::hsla(0.15, 1., 0.5, 0.4)))
            .collect();
        assert_eq!(highlights.len(), 2002);
        state.set_range_highlights(highlights, cx).unwrap();
        state.reveal_range(0..1, cx).unwrap();
    });
    let mut changed = source.clone();
    let edit = source.find("|---|---|\n").unwrap() + "|---|---|\n".len() + "| x | ".len();
    assert_eq!(&source[edit..edit + 1], "x");
    changed.replace_range(edit..edit + 1, "y");
    view.update(cx, |state, cx| state.set_text(&changed, cx));
    // The large full replacement is not committed until the background task runs.
    view.read_with(cx, |state, _| assert_eq!(state.source().as_str(), source));
    cx.run_until_parked();
    view.read_with(cx, |state, _| {
        assert_eq!(state.source().as_str(), changed);
        assert!(state.parsed_error.is_none());
        let frame = state.range_highlights.as_ref().unwrap();
        for ordinal in 0..2002 {
            assert_eq!(frame.backgrounds(TextLeafKey::table_cell(0, ordinal)).len(),
                usize::from(ordinal < 2), "cell {ordinal}");
        }
        assert!(state.pending_reveal.is_some());
    });
}
