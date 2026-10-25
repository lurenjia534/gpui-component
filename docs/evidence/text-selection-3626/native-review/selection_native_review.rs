//! Temporary native-window acceptance fixture for issue 3626.
use gpui_kit::{*, component::{ActiveTheme, Theme, ThemeMode, text::{TextView,TextViewState}}, prelude::FluentBuilder as _};
actions!(selection_review,[RemoveEndpoint,RestoreEndpoint,ProbeSelection,CycleEndpoint]);
struct TextRow { text: Entity<TextViewState> }
impl Render for TextRow {
    fn render(&mut self,_:&mut Window,cx:&mut Context<Self>)->impl IntoElement {
        div().p_3().bg(cx.theme().muted).rounded(cx.theme().radius_lg).child(TextView::new(&self.text))
    }
}
struct Review {
    rows: Vec<Entity<TextRow>>, scrolling:Entity<TextViewState>, focus:FocusHandle, endpoint:bool, phase:u8,
}
impl Render for Review {
    fn render(&mut self,_:&mut Window,cx:&mut Context<Self>)->impl IntoElement {
        div().size_full().bg(cx.theme().background).text_color(cx.theme().foreground).flex().flex_col().p_4().gap_3()
            .track_focus(&self.focus).key_context("SelectionReview")
            .on_action(cx.listener(|this,_:&CycleEndpoint,w,cx|{
                let text=gpui_kit::base::TextSelection::selected_text(w,cx);
                let has=gpui_kit::base::TextSelection::has_selection(w,cx);
                std::fs::write(format!("/home/luren/Desktop/3626-test-results/native-review/unmount-phase-{}.txt",this.phase),format!("has_selection={has}\n{text}")).unwrap();
                if this.phase==0 { this.endpoint=false; }
                if this.phase==1 { this.endpoint=true; }
                this.phase+=1;cx.notify();
            }))
            .on_action(cx.listener(|this,_:&RemoveEndpoint,_,cx|{this.endpoint=false;cx.notify();}))
            .on_action(cx.listener(|this,_:&RestoreEndpoint,_,cx|{this.endpoint=true;cx.notify();}))
            .on_action(cx.listener(|_,_:&ProbeSelection,w,cx|{
                let text=gpui_kit::base::TextSelection::selected_text(w,cx);
                let has=gpui_kit::base::TextSelection::has_selection(w,cx);
                std::fs::write("/home/luren/Desktop/3626-test-results/native-review/native-state.txt",format!("has_selection={has}\n{text}")).unwrap();
            }))
            .child(div().text_xl().child("Issue 3626 native selection review"))
            .child("Ctrl+C record selection then remove CHARLIE then restore then verify")
            .child(self.rows[0].clone()).child(self.rows[1].clone())
            .when(self.endpoint,|this|this.child(self.rows[2].clone()))
            .child("Scroll inside the following document while keeping a selection")
            .child(div().h(px(200.)).overflow_hidden().child(TextView::new(&self.scrolling).scrollable(true)))
    }
}
fn main(){
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx|{
        gpui_kit::init(cx);Theme::change(ThemeMode::Light,None,cx);
        cx.bind_keys([KeyBinding::new("ctrl-c",CycleEndpoint,Some("TextView")),KeyBinding::new("f8",RemoveEndpoint,Some("SelectionReview")),KeyBinding::new("f7",RestoreEndpoint,Some("SelectionReview")),KeyBinding::new("f9",ProbeSelection,Some("SelectionReview"))]);
        let (handle,view)=gpui_kit::open_window(WindowOptions{window_bounds:Some(WindowBounds::centered(size(px(900.),px(700.)),cx)),..Default::default()},cx,|w,cx|{
            w.set_window_title("Issue 3626 native selection review");
            cx.new(|cx|{
                let focus=cx.focus_handle();focus.focus(w,cx);
                Review{rows:["ALPHA first selectable paragraph", "BRAVO second selectable paragraph", "CHARLIE third selectable paragraph"].iter().map(|s|cx.new(|cx|TextRow{text:cx.new(|cx|TextViewState::markdown(s,cx))})).collect(),scrolling:cx.new(|cx|TextViewState::markdown(&(1..=30).map(|n|format!("Scrollable row {n:02} alpha bravo charlie")).collect::<Vec<_>>().join("\n\n"),cx)),focus,endpoint:true,phase:0}
            })
        }).unwrap();
        handle.update(cx,|_,w,cx|{let focus=view.read(cx).focus.clone();focus.focus(w,cx)}).unwrap();
    });
}
