use gpui_kit::assets::Assets;
use gpui_kit::component::{
    input::{Copy, Input, InputState, Textarea, TextareaState},
    menu::ContextMenuExt,
    ActiveTheme, StyledExt, v_flex,
};
use gpui_kit::*;

struct SelectionDemo {
    input: Entity<InputState>,
    textarea: Entity<TextareaState>,
    destination: Entity<InputState>,
}

impl SelectionDemo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value("GPUI Kit selection stays visible while the editing menu is open.")
            }),
            textarea: cx.new(|cx| {
                TextareaState::new(window, cx)
                    .context_menu(false)
                    .default_value("GPUI Kit selection stays visible.\nCopy this selected text using the context menu.")
            }),
            destination: cx.new(|cx| InputState::new(window, cx).placeholder("Paste copied text here")),
        }
    }
}

impl Render for SelectionDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = self.textarea.focus_handle(cx);
        v_flex()
            .size_full()
            .p_8()
            .gap_6()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(div().text_lg().child("GPUI Kit · #3336 selection highlight"))
            .child(
                v_flex()
                    .gap_2()
                    .child("Input · default editing menu")
                    .child(Input::new(&self.input)),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child("Textarea · ContextMenu with keyboard navigation")
                    .child(
                        div()
                            .id("selection-menu")
                            .child(Textarea::new(&self.textarea).h(px(140.)))
                            .context_menu(move |menu, window, cx| {
                                menu.action_context(target.clone())
                                    .menu("Copy", Box::new(Copy))
                                    .submenu("More", window, cx, |menu, _, _| {
                                        menu.menu("Copy", Box::new(Copy))
                                    })
                            }),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child("Another input · keyboard and clipboard check")
                    .child(Input::new(&self.destination)),
            )
    }
}

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(920.), px(620.)), cx)),
                ..Default::default()
            },
            cx,
            |window, cx| {
                window.set_window_title("GPUI Kit #3336 native selection test");
                cx.new(|cx| SelectionDemo::new(window, cx))
            },
        )
        .expect("Failed to open the native test window");
        cx.activate(true);
    });
}
