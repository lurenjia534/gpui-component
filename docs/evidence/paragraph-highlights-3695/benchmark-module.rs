
#[cfg(test)]
mod issue_3695_benchmark {
    use super::*;
    impl Paragraph {
    fn legacy_plain_render(
        &self,
        node_cx: &NodeContext,
        cx: &App,
    ) -> (
        SharedString,
        Vec<(Range<usize>, InlineHighlight)>,
        Vec<(Range<usize>, LinkMark)>,
    ) {
        let mono_font = cx.theme().tokens.typography.mono.clone();
        let background = cx.theme().tokens.colors.background;
        if let Ok(cache) = self.render_cache.0.lock()
            && let Some(cached) = cache.as_ref()
            && (Arc::ptr_eq(&cached.style, &node_cx.style) || *cached.style == *node_cx.style)
            && cached.mono_font == mono_font
            && cached.background == background
        {
            return (
                cached.text.clone(),
                foreground_highlights(
                    cached.highlights.clone(),
                    &cached.text,
                    0,
                    node_cx.foregrounds(self.leaf_key()),
                ),
                cached.links.clone(),
            );
        }

        let mut text = String::new();
        let mut highlights: Vec<(Range<usize>, InlineHighlight)> = vec![];
        let mut links: Vec<(Range<usize>, LinkMark)> = vec![];
        let mut offset = 0;
        for inline_node in &self.children {
            let text_len = inline_node.text.len();
            text.push_str(&inline_node.text);
            let mut node_highlights = vec![];
            for (range, style) in &inline_node.marks {
                let inner_range = (offset + range.start)..(offset + range.end);
                let mut highlight = mark_highlight(style, node_cx, cx);
                if let Some(link_mark) = style.link.clone() {
                    highlight.style.color = Some(node_cx.style.link());
                    highlight.style.underline = Some(link_underline(&node_cx.style));
                    links.push((inner_range.clone(), link_mark));
                }
                node_highlights.push((inner_range, highlight));
            }
            highlights = combine_highlights(highlights, node_highlights);
            offset += text_len;
        }
        let text = SharedString::from(text);
        if let Ok(mut cache) = self.render_cache.0.lock() {
            *cache = Some(Arc::new(ParagraphRender {
                style: node_cx.style.clone(),
                mono_font,
                background,
                text: text.clone(),
                highlights: highlights.clone(),
                links: links.clone(),
            }));
        }
        let highlights =
            foreground_highlights(highlights, &text, 0, node_cx.foregrounds(self.leaf_key()));
        (text, highlights, links)
    }

    fn legacy_inline_highlights(
        &self,
        node_cx: &NodeContext,
        cx: &App,
    ) -> Vec<(Range<usize>, InlineHighlight)> {
        let mut highlights = vec![];
        let mut offset = 0;
        for inline_node in &self.children {
            let node_highlights = inline_node
                .marks
                .iter()
                .map(|(range, mark)| {
                    (
                        (offset + range.start)..(offset + range.end),
                        mark_highlight(mark, node_cx, cx),
                    )
                })
                .collect::<Vec<_>>();
            highlights = combine_highlights(highlights, node_highlights);
            offset += inline_node.text.len();
        }
        highlights
    }

    fn legacy_inline_flow_items(
        &self,
        leaf_key: Option<TextLeafKey>,
        fades: &[(Range<usize>, f32)],
        backgrounds: &[(Range<usize>, Hsla)],
        node_cx: &NodeContext,
        cx: &mut App,
    ) -> Vec<InlineFlowItem> {
        let mut items = Vec::new();
        let mut text = String::new();
        let mut highlights: Vec<(Range<usize>, InlineHighlight)> = vec![];
        let mut links: Vec<(Range<usize>, LinkMark)> = vec![];
        let mut offset = 0;
        // Where `text` starts in the paragraph's whole rendered text, which
        // is the byte space the fade and background ranges use.
        let mut consumed = 0;

        for inline_node in &self.children {
            if let Some(node) = &inline_node.custom {
                if let Ok(mut state) = inline_node.state.lock() {
                    state.set_text(text.clone().into());
                }
                if !text.is_empty() {
                    let item_fades = slice_fades(fades, consumed, consumed + text.len());
                    let item_backgrounds =
                        slice_backgrounds(backgrounds, consumed, consumed + text.len());
                    let item_reveal = node_cx.reveal_at(leaf_key, consumed, consumed + text.len());
                    consumed += text.len();
                    items.push(InlineFlowItem::Text {
                        state: inline_node.state.clone(),
                        text: std::mem::take(&mut text).into(),
                        links: std::mem::take(&mut links),
                        highlights: fade_highlights(std::mem::take(&mut highlights), &item_fades),
                        backgrounds: item_backgrounds,
                        reveal: item_reveal,
                    });
                }
                let mut object_style = HighlightStyle::default();
                let mut object_link = None;
                for (_, mark) in &inline_node.marks {
                    object_style = object_style.highlight(mark_highlight(mark, node_cx, cx).style);
                    if let Some(link) = &mark.link {
                        object_link = Some(
                            link.identifier
                                .as_ref()
                                .and_then(|id| node_cx.link_refs.get(id))
                                .unwrap_or(link)
                                .clone(),
                        );
                        object_style.color = Some(node_cx.style.link());
                        object_style.underline = Some(link_underline(&node_cx.style));
                    }
                }
                let rendered_node = node.clone();
                let extensions = node_cx.markdown_extensions.clone();
                items.push(InlineFlowItem::Object {
                    text: node.shared_text(),
                    accessibility_label: node.shared_accessibility_name(),
                    id: node.source_range().map_or(items.len(), |range| range.start),
                    renderer: Arc::new(move |context, window, cx| {
                        extensions.render_inline(&rendered_node, context, window, cx)
                    }),
                    selected: inline_node.custom_selection.clone(),
                    style: object_style,
                    link: object_link,
                });
                consumed += inline_node.text.len();
                offset = 0;
                continue;
            }
            let text_len = inline_node.text.len();
            text.push_str(&inline_node.text);

            if let Some(image) = &inline_node.image {
                if !text.is_empty() {
                    if let Ok(mut state) = inline_node.state.lock() {
                        state.set_text(text.clone().into());
                    }
                    items.push(InlineFlowItem::Text {
                        state: inline_node.state.clone(),
                        text: text.clone().into(),
                        links: links.clone(),
                        highlights: fade_highlights(
                            highlights.clone(),
                            &slice_fades(fades, consumed, consumed + text.len()),
                        ),
                        backgrounds: slice_backgrounds(
                            backgrounds,
                            consumed,
                            consumed + text.len(),
                        ),
                        reveal: node_cx.reveal_at(leaf_key, consumed, consumed + text.len()),
                    });
                }

                items.push(InlineFlowItem::Image {
                    source: node_cx.image_source(image),
                    link: image.link.clone(),
                    title: image.title(),
                    width: image.width,
                    height: image.height,
                });

                consumed += text.len();
                text.clear();
                links.clear();
                highlights.clear();
                offset = 0;
            } else {
                let mut node_highlights = vec![];
                for (range, style) in &inline_node.marks {
                    let inner_range = (offset + range.start)..(offset + range.end);
                    let mut highlight = mark_highlight(style, node_cx, cx);

                    if let Some(mut link_mark) = style.link.clone() {
                        highlight.style.color = Some(node_cx.style.link());
                        highlight.style.underline = Some(link_underline(&node_cx.style));

                        if let Some(identifier) = link_mark.identifier.as_ref()
                            && let Some(mark) = node_cx.link_refs.get(identifier)
                        {
                            link_mark = mark.clone();
                        }

                        links.push((inner_range.clone(), link_mark));
                    }

                    node_highlights.push((inner_range, highlight));
                }

                highlights = combine_highlights(highlights, node_highlights);
                offset += text_len;
            }
        }

        if !text.is_empty() {
            if let Ok(mut state) = self.state.lock() {
                state.set_text(text.clone().into());
            }
            let highlights = fade_highlights(
                highlights,
                &slice_fades(fades, consumed, consumed + text.len()),
            );
            let backgrounds = slice_backgrounds(backgrounds, consumed, consumed + text.len());
            let reveal = node_cx.reveal_at(leaf_key, consumed, consumed + text.len());
            items.push(InlineFlowItem::Text {
                state: self.state.clone(),
                text: text.into(),
                links,
                highlights,
                backgrounds,
                reveal,
            });
        }

        let foregrounds = node_cx.foregrounds(self.leaf_key());
        if !foregrounds.is_empty() {
            let mut offset = 0;
            for item in &mut items {
                match item {
                    InlineFlowItem::Text {
                        text, highlights, ..
                    } => {
                        *highlights = foreground_highlights(
                            std::mem::take(highlights),
                            text,
                            offset,
                            foregrounds,
                        );
                        offset += text.len();
                    }
                    InlineFlowItem::Object { style, .. } => {
                        let end = offset + '\u{FFFC}'.len_utf8();
                        if let Some((_, color)) = foregrounds
                            .iter()
                            .rev()
                            .find(|(range, _)| range.start <= offset && end <= range.end)
                        {
                            style.color = Some(*color);
                        }
                        offset = end;
                    }
                    InlineFlowItem::Image { .. } => offset += '\u{FFFC}'.len_utf8(),
                }
            }
        }
        items
    }
    }

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

}
