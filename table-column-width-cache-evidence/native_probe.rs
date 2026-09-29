//! Native present-cadence probe. Kept outside production sources.
use gpui::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, Overflow,
    ParentElement as _, Render, ScrollHandle, StatefulInteractiveElement as _, StyleRefinement,
    Styled as _, WeakEntity, Window, WindowBounds, WindowOptions, div, point,
    profiler::{self, FrameEvent, FrameTiming, FrameTimingCollector, PresentTiming},
    px, size,
};
use gpui_base::text::{TextView, TextViewState, TextViewStyle};
use std::fmt::Write as _;

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
        "small" => Fixture {
            name: "small",
            tables: 1,
            rows: 8,
            cols: 4,
            long: false,
            scroll: true,
        },
        "medium" => Fixture {
            name: "medium",
            tables: 1,
            rows: 48,
            cols: 6,
            long: false,
            scroll: true,
        },
        "large" => Fixture {
            name: "large",
            tables: 1,
            rows: 200,
            cols: 8,
            long: false,
            scroll: true,
        },
        "long" => Fixture {
            name: "long",
            tables: 1,
            rows: 8,
            cols: 4,
            long: true,
            scroll: true,
        },
        "many" => Fixture {
            name: "many",
            tables: 12,
            rows: 8,
            cols: 4,
            long: false,
            scroll: true,
        },
        "wrap" => Fixture {
            name: "wrap",
            tables: 1,
            rows: 48,
            cols: 6,
            long: false,
            scroll: false,
        },
        _ => panic!("unknown fixture {name}"),
    }
}

fn source(f: Fixture, revision: usize) -> String {
    let mut out = String::new();
    for table in 0..f.tables {
        writeln!(out, "Summary {table}: revision {revision}\n").unwrap();
        for col in 0..f.cols {
            write!(out, "| Field {col} ").unwrap();
        }
        out.push_str("|\n");
        for _ in 0..f.cols {
            out.push_str("| --- ");
        }
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
                if f.long {
                    out.push_str(&"Measured text 数据内容 ".repeat(48));
                }
            }
            out.push_str("|\n");
        }
        out.push_str("\n");
    }
    out
}

struct Probe {
    view: Entity<TextViewState>,
    fixture: Fixture,
    mode: String,
    scroll: ScrollHandle,
    collector: FrameTimingCollector,
    warmup: usize,
    goal: usize,
    frame_ix: usize,
    inactive_samples: usize,
    draws: Vec<FrameTiming>,
    presents: Vec<PresentTiming>,
    refresh_hz: f64,
}

impl Probe {
    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for event in self.collector.collect_unseen() {
            match event {
                FrameEvent::Draw(timing)
                    if timing.window_id == window.window_handle().window_id() =>
                {
                    if self.warmup == 0 {
                        self.draws.push(timing);
                    }
                }
                FrameEvent::Present(timing)
                    if timing.window_id == window.window_handle().window_id() =>
                {
                    if self.warmup > 0 {
                        self.warmup -= 1;
                    } else {
                        self.presents.push(timing);
                        if !window.is_window_active() {
                            self.inactive_samples += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        if self.presents.len() >= self.goal {
            self.finish(window, cx);
            cx.quit();
            return;
        }
        if self.mode == "scroll" {
            let maximum = f32::from(self.scroll.max_offset().y);
            if maximum > 0. {
                self.scroll
                    .set_offset(point(px(0.), px(-((self.frame_ix as f32 * 37.) % maximum))));
            }
        }
        self.frame_ix += 1;
        cx.notify();
        schedule(cx.entity().downgrade(), window);
    }

    fn finish(&self, window: &mut Window, cx: &mut Context<Self>) {
        assert!(self.presents.len() >= 2);
        assert_eq!(
            self.draws.len(),
            self.presents.len(),
            "draw/present count mismatch"
        );
        let first = self.presents.first().unwrap().present_end;
        let last = self.presents.last().unwrap().present_end;
        let intervals: Vec<f64> = self
            .presents
            .windows(2)
            .map(|p| {
                p[1].present_end
                    .duration_since(p[0].present_end)
                    .as_secs_f64()
                    * 1000.
            })
            .collect();
        let draw_ms: Vec<f64> = self
            .draws
            .iter()
            .map(|d| d.draw_duration().as_secs_f64() * 1000.)
            .collect();
        let submit_ms: Vec<f64> = self
            .presents
            .iter()
            .map(|p| p.present_duration().as_secs_f64() * 1000.)
            .collect();
        let timestamps_ms: Vec<f64> = self
            .presents
            .iter()
            .map(|p| p.present_end.duration_since(first).as_secs_f64() * 1000.)
            .collect();
        let source_bytes = source(self.fixture, 0).len();
        let rendered = self.view.read(cx).rendered_text();
        for table in 0..self.fixture.tables {
            assert!(
                rendered.as_str().contains(&format!(
                    "Service {table}-{}: storage",
                    self.fixture.rows - 1
                )),
                "the table's final row must have finished parsing"
            );
        }
        if self.mode == "scroll" {
            assert!(self.scroll.max_offset().y > px(0.));
        }
        println!(
            "NATIVE_JSON {}",
            serde_json::json!({
                "case": self.fixture.name, "mode": self.mode,
                "warmup_presents": 120, "draw_ms": draw_ms, "submit_ms": submit_ms,
                "present_intervals_ms": intervals, "present_timestamps_ms": timestamps_ms,
                "present_fps": (self.presents.len() - 1) as f64 / last.duration_since(first).as_secs_f64(),
                "declared_refresh_hz": self.refresh_hz,
                "inactive_samples": self.inactive_samples, "source_bytes": source_bytes,
                "window_bounds": format!("{:?}", window.bounds()),
                "document_bounds": format!("{:?}", self.view.read(cx).bounds()),
                "scroll_max": format!("{:?}", self.scroll.max_offset()),
                "display": format!("{:?}", window.display(cx)), "scale_factor": window.scale_factor(),
            "gpu": window.gpu_specs(),
                "backend": std::env::var("WAYLAND_DISPLAY").ok().map(|_| "wayland").unwrap_or("x11"),
            })
        );
    }
}

fn schedule(entity: WeakEntity<Probe>, window: &Window) {
    window.on_next_frame(move |window, cx| {
        let _ = entity.update(cx, |probe, cx| probe.tick(window, cx));
    });
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut table = StyleRefinement::default();
        if self.fixture.scroll {
            table.overflow.x = Some(Overflow::Scroll);
        }
        let colors = gpui_base::Theme::global(cx).tokens.colors;
        div()
            .id("evidence-viewport")
            .w(px(800.))
            .h(px(600.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(colors.background)
            .text_color(colors.foreground)
            .font_family("DejaVu Sans")
            .text_size(px(14.))
            .child(TextView::new(&self.view).style(TextViewStyle::default().with_table(table)))
    }
}

fn main() {
    let f = fixture(&std::env::var("WIDTH_CASE").unwrap_or_else(|_| "large".into()));
    let mode = std::env::var("WIDTH_MODE").unwrap_or_else(|_| "scroll".into());
    assert!(matches!(mode.as_str(), "redraw" | "scroll"));
    let goal: usize = std::env::var("WIDTH_PRESENTS")
        .unwrap_or_else(|_| "600".into())
        .parse()
        .unwrap();
    let refresh_hz: f64 = std::env::var("WIDTH_REFRESH_HZ")
        .expect("Set WIDTH_REFRESH_HZ to the active display's configured refresh rate")
        .parse()
        .unwrap();
    assert!(goal >= 2 && refresh_hz > 0.);
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_base::init(cx);
        gpui_base::Theme::global_mut(cx).tokens.typography.mono = "DejaVu Sans Mono".into();
        profiler::set_trace_enabled(true);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(800.), px(600.)), cx)),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("Column width cache measurement");
                let view = cx.new(|cx| TextViewState::markdown(&source(f, 0), cx));
                let entity = cx.new(|_| Probe {
                    view,
                    fixture: f,
                    mode,
                    scroll: ScrollHandle::new(),
                    collector: FrameTimingCollector::new(),
                    warmup: 120,
                    goal,
                    frame_ix: 0,
                    inactive_samples: 0,
                    draws: Vec::with_capacity(goal),
                    presents: Vec::with_capacity(goal),
                    refresh_hz,
                });
                schedule(entity.downgrade(), window);
                window.activate_window();
                entity
            },
        )
        .expect("open native benchmark window");
        cx.activate(true);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
    });
}
