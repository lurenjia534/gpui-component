//! Temporary evidence fixture. Run with --features gpui-fast,test-support.
use gpui_kit::{*, component::{ActiveTheme, Theme, ThemeMode, text::{TextView, TextViewState}}, test::{TestSupportExt, TestWindowExt}};
use std::{cell::Cell, rc::Rc, sync::Arc, time::Instant};

struct Tile { id: usize, text: Entity<TextViewState>, renders: Rc<Cell<usize>> }
impl Render for Tile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div().id(("tile",self.id)).test_support().w(px(280.)).h(px(30.)).px_2()
            .child(TextView::new(&self.text)).text_color(cx.theme().foreground)
    }
}
struct Page { tiles: Vec<Entity<Tile>>, count: usize }
impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(cx.theme().background).text_color(cx.theme().foreground).p_4().flex().flex_col().gap_2()
            .child(div().text_xl().child("Text selection performance comparison"))
            .child(div().child(format!("{} selectable TextViews | GPUI Fast | drag within text 000", self.count)))
            .child(div().text_color(cx.theme().muted_foreground).child("The selected text should change. Other text views should remain retained."))
            .child(div().flex().flex_wrap().w(px(1120.)).children(self.tiles.iter().cloned()))
    }
}
fn draw(cx: &mut HeadlessAppContext, handle: AnyWindowHandle) {
    cx.update_window(handle, |_,w,cx| w.draw(cx).clear(cx)).unwrap();
    cx.run_until_parked();
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let label = args.get(1).expect("before or after");
    let output = args.get(2).expect("output directory");
    println!("version,count,trial,sample,input_draw_deferred_us,total_renders,unrelated_renders");
    for count in [16,64,164] {
        for trial in 0..3 {
            let mut cx = HeadlessAppContext::with_platform(gpui_kit::platform::current_platform(true).text_system(), Arc::new(gpui_kit::assets::Assets), gpui_kit::platform::current_headless_renderer);
            cx.update(|cx| { gpui_kit::init(cx); Theme::change(ThemeMode::Light,None,cx); });
            let counters: Vec<_> = (0..count).map(|_|Rc::new(Cell::new(0))).collect();
            let (handle, _) = cx.update(|cx| gpui_kit::open_window(WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds{ origin:point(px(0.),px(0.)), size:size(px(1180.),px(1400.))})), focus:false, show:false, ..Default::default()
            }, cx, |_,cx| cx.new(|cx| Page {
                count, tiles:(0..count).map(|id| cx.new(|cx| Tile { id, renders:counters[id].clone(), text:cx.new(|cx|TextViewState::markdown(&format!("Text {id:03} alpha bravo charlie"),cx)) })).collect()
            }))).unwrap();
            let handle=handle.into();
            cx.run_until_parked();
            for _ in 0..4 { draw(&mut cx,handle); }
            let bounds=cx.update_window(handle,|_,w,_| w.find(("tile",0usize)).bounds()).unwrap();
            let start=point(bounds.left()+px(9.),bounds.top()+px(12.));
            cx.update_window(handle,|_,w,cx| w.dispatch_event(PlatformInput::MouseDown(MouseDownEvent {position:start,button:MouseButton::Left,click_count:1,..Default::default()}),cx)).unwrap();
            for sample in 0..70 {
                let fraction = ((sample % 20) as f32 / 19. - 0.5).abs() * 2.;
                let end=point(bounds.left()+px(45.+fraction*150.), start.y);
                let total_before:usize=counters.iter().map(|c|c.get()).sum();
                let unrelated_before:usize=counters[1..].iter().map(|c|c.get()).sum();
                let time=Instant::now();
                cx.update_window(handle,|_,w,cx|w.dispatch_event(PlatformInput::MouseMove(MouseMoveEvent {position:end,pressed_button:Some(MouseButton::Left),..Default::default()}),cx)).unwrap();
                draw(&mut cx,handle);
                let elapsed=time.elapsed().as_secs_f64()*1e6;
                let total:usize=counters.iter().map(|c|c.get()).sum();
                let unrelated:usize=counters[1..].iter().map(|c|c.get()).sum();
                if sample>=10 { println!("{label},{count},{trial},{},{elapsed:.3},{},{}",sample-10,total-total_before,unrelated-unrelated_before); }
            }
            let copied=cx.update_window(handle,|_,w,cx|gpui_kit::base::TextSelection::selected_text(w,cx)).unwrap();
            assert!(!copied.trim().is_empty(),"gesture must select real text");
            eprintln!("{label} count={count} trial={trial} selected={copied:?}");
            if count==64 && trial==0 {
                cx.capture_screenshot(handle).expect("real GPU renderer required").save(format!("{output}/{label}.png")).unwrap();
            }
        }
    }
}
