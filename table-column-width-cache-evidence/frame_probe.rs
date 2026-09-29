//! Temporary evidence probe, included as a child of text::node in both builds.
use super::*;
use crate::text::{TextView, TextViewState};
use gpui::{AppContext as _, Context, Entity, Render, ScrollHandle, TestApp, TestAppWindow, point};
use std::{fmt::Write as _, hint::black_box, time::Instant};

#[derive(Clone, Copy)]
struct Fixture {
    name: &'static str,
    tables: usize,
    rows: usize,
    cols: usize,
    long: bool,
    scroll: bool,
}

fn fixture(name: &str) -> Fixture {
    match name {
        "small" => Fixture { name: "small", tables: 1, rows: 8, cols: 4, long: false, scroll: true },
        "medium" => Fixture { name: "medium", tables: 1, rows: 48, cols: 6, long: false, scroll: true },
        "large" => Fixture { name: "large", tables: 1, rows: 200, cols: 8, long: false, scroll: true },
        "long" => Fixture { name: "long", tables: 1, rows: 8, cols: 4, long: true, scroll: true },
        "many" => Fixture { name: "many", tables: 12, rows: 8, cols: 4, long: false, scroll: true },
        "wrap" => Fixture { name: "wrap", tables: 1, rows: 48, cols: 6, long: false, scroll: false },
        _ => panic!("unknown fixture {name}"),
    }
}

fn source(f: Fixture, revision: usize) -> String {
    let mut out = String::new();
    for table in 0..f.tables {
        writeln!(out, "Summary {table}: revision {revision}\n").unwrap();
        for col in 0..f.cols { write!(out, "| Field {col} ").unwrap(); }
        out.push_str("|\n");
        for _ in 0..f.cols { out.push_str("| --- "); }
        out.push_str("|\n");
        for row in 0..f.rows {
            for col in 0..f.cols {
                let text = match col % 4 {
                    0 => format!("Service {table}-{row}: storage"),
                    1 => format!("状态正常，处理 {} 个请求", 1200 + row),
                    2 => format!("`request_{row}_{revision}`"),
                    _ => format!("{:.2} ms, **healthy**", 1.5 + row as f64 / 10.),
                };
                write!(out, "| {text} ").unwrap();
                if f.long { out.push_str(&"Measured text 数据内容 ".repeat(48)); }
            }
            out.push_str("|\n");
        }
        out.push_str("\n");
    }
    out
}

struct Page {
    view: Option<Entity<TextViewState>>,
    scroll: ScrollHandle,
    width: f32,
    font_size: f32,
    table_scroll: bool,
    renders: usize,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders += 1;
        let mut table = StyleRefinement::default();
        if self.table_scroll { table.overflow.x = Some(Overflow::Scroll); }
        div()
            .id("evidence-viewport")
            .w(px(self.width))
            .h(px(600.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .font_family("DejaVu Sans")
            .text_size(px(self.font_size))
            .when_some(self.view.as_ref(), |element, view| {
                element.child(TextView::new(view).style(TextViewStyle::default().with_table(table)))
            })
    }
}

fn app() -> TestApp {
    let platform = gpui_platform::current_platform(true);
    let mut app = TestApp::with_text_system(platform.text_system());
    app.update(|cx| {
        crate::init(cx);
        crate::Theme::global_mut(cx).tokens.typography.mono = "DejaVu Sans Mono".into();
    });
    app
}

fn page(app: &mut TestApp, scroll: bool) -> TestAppWindow<Page> {
    app.open_window(|_, _| Page {
        view: None,
        scroll: ScrollHandle::new(),
        width: 800.,
        font_size: 14.,
        table_scroll: scroll,
        renders: 0,
    })
}

fn frame(app: &mut TestApp, page: &mut TestAppWindow<Page>, mode: &str, ix: usize, replacement: &str) -> f64 {
    // Parse replacements while detached. TestAppWindow::update runs the executor
    // and can draw automatically, so mounting/mutation and the timed draw must
    // happen in a single App update, before returning to that executor.
    let next_view = (mode == "replace" || mode == "first")
        .then(|| app.new_entity(|cx| TextViewState::markdown(replacement, cx)));
    app.run_until_parked();
    app.update(|cx| cx.update_window(page.handle().into(), |root, window, cx| {
        let root = root.downcast::<Page>().unwrap();
        let before = root.read(cx).renders;
        root.update(cx, |page, _| { match mode {
            "scroll" => {
                let max = f32::from(page.scroll.max_offset().y);
                let offset = if max > 0. { (ix as f32 * 37.) % max } else { 0. };
                page.scroll.set_offset(point(px(0.), px(-offset)));
            }
            "resize" => page.width = if ix % 2 == 0 { 640. } else { 960. },
            "typography" => page.font_size = if ix % 2 == 0 { 14. } else { 18. },
            "replace" | "first" => page.view = next_view,
            "release" => page.view = None,
            "redraw" => {},
            _ => panic!("unknown mode"),
        }});
        window.refresh();
        let started = Instant::now();
        window.draw(cx).clear(cx);
        let elapsed = started.elapsed().as_secs_f64() * 1e6;
        assert_eq!(root.read(cx).renders, before + 1);
        elapsed
    }).unwrap())
}

fn memory() -> serde_json::Value {
    #[repr(C)]
    struct Mallinfo {
        arena: usize, ordblks: usize, smblks: usize, hblks: usize, hblkhd: usize,
        usmblks: usize, fsmblks: usize, uordblks: usize, fordblks: usize, keepcost: usize,
    }
    unsafe extern "C" { fn mallinfo2() -> Mallinfo; }
    let stats = unsafe { mallinfo2() };
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let rss: usize = status.lines().find(|line| line.starts_with("VmRSS:")).unwrap()
        .split_whitespace().nth(1).unwrap().parse().unwrap();
    serde_json::json!({"rss_kib":rss,"heap_used":stats.uordblks,"mmap_bytes":stats.hblkhd})
}

#[test]
#[ignore]
fn frames() {
    let f = fixture(&std::env::var("WIDTH_CASE").unwrap());
    let mode = std::env::var("WIDTH_MODE").unwrap_or_else(|_| "redraw".into());
    let count: usize = std::env::var("WIDTH_FRAMES").unwrap_or_else(|_| "120".into()).parse().unwrap();
    let source = source(f, 0);
    let replacement = self::source(f, 1);
    let mut app = app();
    let before = memory();
    let mut page = page(&mut app, f.scroll);
    app.run_until_parked();
    assert!(page.read(|page, _| page.view.is_none()));
    let first_us = frame(&mut app, &mut page, "first", 0, &source);
    let first_memory = memory();
    let geometry = page.read(|page, cx| {
        let view = page.view.as_ref().unwrap().read(cx);
        assert_eq!(view.source().as_ref(), source.as_str());
        assert!(f32::from(view.bounds().size.height) > 0.);
        serde_json::json!({"bounds":format!("{:?}",view.bounds()),"scroll_max":format!("{:?}",page.scroll.max_offset())})
    });
    for ix in 0..20 { frame(&mut app, &mut page, &mode, ix, if ix % 2 == 0 { &replacement } else { &source }); }
    let before_renders = page.read(|page, _| page.renders);
    let samples: Vec<f64> = (0..count).map(|ix| frame(&mut app, &mut page, &mode, ix + 20,
        if ix % 2 == 0 { &replacement } else { &source })).collect();
    assert_eq!(page.read(|page, _| page.renders) - before_renders, count);
    let steady_memory = memory();
    frame(&mut app, &mut page, "release", 0, "");
    frame(&mut app, &mut page, "redraw", 0, "");
    app.run_until_parked();
    let released_memory = memory();
    println!("WIDTH_JSON {}", serde_json::json!({"case":f.name,"mode":mode,"source_bytes":source.len(),
        "first_us":first_us,"samples_us":samples,"before":before,"first_memory":first_memory,
        "steady_memory":steady_memory,"released_memory":released_memory,"geometry":geometry,
        "table_bytes":std::mem::size_of::<Table>()}));
}

#[test]
#[ignore]
fn retained_tables() {
    use crate::text::inline::test_draw::in_prepaint;
    let count: usize = std::env::var("WIDTH_TABLES").unwrap_or_else(|_| "1000".into()).parse().unwrap();
    let cols: usize = std::env::var("WIDTH_COLS").unwrap_or_else(|_| "4".into()).parse().unwrap();
    let mut app = app();
    in_prepaint(&mut app, move |window, cx| {
        let context = NodeContext::default();
        let template = Table { children: vec![TableRow { children: (0..cols).map(|col| TableCell {
            children: Paragraph { children: vec![InlineNode::new(format!("column {col} ordinary text"))], ..Default::default() },
            width: None,
        }).collect() }], ..Default::default() };
        // Warm the platform font and shaped-line caches without measuring the retained tables.
        black_box(measure_table_columns(&template, cols, &context, window, cx));
        let before = memory();
        let tables: Vec<Table> = (0..count).map(|_| Table {
            children: template.children.clone(), ..Default::default()
        }).collect();
        let allocated = memory();
        for table in &tables { black_box(measure_table_columns(table, cols, &context, window, cx)); }
        let measured = memory();
        for _ in 0..10 { for table in &tables { black_box(measure_table_columns(table, cols, &context, window, cx)); } }
        let repeated = memory();
        #[cfg(feature = "width-cache-present")]
        let exact = {
            let cache = tables[0].column_widths_cache.0.lock().unwrap().clone().unwrap();
            serde_json::json!({"cache_value_bytes":std::mem::size_of::<TableColumnWidths>(),
                "width_capacity":cache.widths.capacity(),"custom_capacity":cache.custom_cells.capacity(),
                "payload_plus_arc_bytes":std::mem::size_of::<TableColumnWidths>() + 2 * std::mem::size_of::<usize>() + cache.widths.capacity() * 4 + cache.custom_cells.capacity() * std::mem::size_of::<(usize,usize)>()})
        };
        #[cfg(not(feature = "width-cache-present"))]
        let exact = serde_json::Value::Null;
        drop(tables);
        let released = memory();
        println!("WIDTH_JSON {}",serde_json::json!({"tables":count,"cols":cols,"before":before,"allocated":allocated,
            "measured":measured,"repeated":repeated,"released":released,"table_bytes":std::mem::size_of::<Table>(),"exact":exact}));
    });
}

#[test]
#[ignore]
#[cfg(feature = "width-cache-present")]
fn cache_lifecycle() {
    use crate::text::inline::test_draw::in_prepaint;
    let mut app = app();
    in_prepaint(&mut app, move |window, cx| {
        let context = NodeContext::default();
        let table = Table { children: vec![TableRow { children: vec![TableCell {
            children: Paragraph { children: vec![InlineNode::new("retained table")], ..Default::default() },
            width: None,
        }] }], ..Default::default() };
        measure_table_columns(&table, 1, &context, window, cx);
        let first = Arc::downgrade(table.column_widths_cache.0.lock().unwrap().as_ref().unwrap());
        let cloned = table.clone();
        assert_eq!(first.strong_count(), 2);
        drop(cloned);
        assert_eq!(first.strong_count(), 1);
        let mut previous = first;
        for ix in 0..1000 {
            window.set_rem_size(px(if ix % 2 == 0 { 18. } else { 16. }));
            measure_table_columns(&table, 1, &context, window, cx);
            assert!(previous.upgrade().is_none(), "replaced cache retained its allocation");
            previous = Arc::downgrade(table.column_widths_cache.0.lock().unwrap().as_ref().unwrap());
        }
        drop(table);
        assert!(previous.upgrade().is_none(), "table drop retained its cache");
        println!("WIDTH_JSON {}",serde_json::json!({"lifecycle":"passed","invalidations":1000,"clone_shares_cache":true,"drop_releases_cache":true}));
    });
}
