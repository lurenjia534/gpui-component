//! Review fixture for #3468. The inspector reads real rendered-node metadata.
//! `fixed` enables the new public API; the baseline cannot compile that API.
use gpui_kit::{
    AppContext, Context, Entity, HeadlessAppContext, IntoElement, Render, Role, SharedString,
    Window, WindowBounds, WindowOptions,
    assets::Assets,
    base::{Slider as BaseSlider, SliderIndicator, SliderThumb, SliderTrack, StyledExt as _},
    component::{
        ActiveTheme, Theme, ThemeMode,
        slider::{Slider, SliderState},
    },
    div,
    prelude::*,
    px, relative, size,
    test::{TestQueryExt, TestWindowExt},
};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};

#[cfg(feature = "fixed")]
fn name_base(slider: BaseSlider, label: Option<&'static str>) -> BaseSlider {
    slider.when_some(label, |slider, label| slider.accessibility_label(label))
}

#[cfg(not(feature = "fixed"))]
fn name_base(slider: BaseSlider, _: Option<&'static str>) -> BaseSlider {
    slider
}

#[cfg(feature = "fixed")]
fn name_component(slider: Slider, label: Option<&'static str>) -> Slider {
    slider.when_some(label, |slider, label| slider.accessibility_label(label))
}

#[cfg(not(feature = "fixed"))]
fn name_component(slider: Slider, _: Option<&'static str>) -> Slider {
    slider
}

#[derive(Clone)]
struct Observation {
    layer: &'static str,
    intended_name: &'static str,
    role: String,
    label: Option<String>,
    path: String,
    state_value: f32,
    bounds: [f32; 4],
}

impl Observation {
    fn json(&self) -> Value {
        json!({
            "layer": self.layer,
            "intended_name": self.intended_name,
            "role": self.role,
            "label": self.label,
            "element_identity_path": self.path,
            "state_value": self.state_value,
            "bounds": self.bounds,
        })
    }
}

struct Review {
    base: [Entity<SliderState>; 2],
    component: [Entity<SliderState>; 2],
    labels: Option<[&'static str; 2]>,
    observations: Option<Vec<Observation>>,
}

impl Review {
    fn base_slider(
        &self,
        state: &Entity<SliderState>,
        label: Option<&'static str>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let percentage = state.read(cx).percentage().end;
        // The geometry follows the published Base Slider showcase. Presentation
        // belongs to this fixture, and its colors come from the Component theme.
        BaseSlider::new(state)
            .map(|slider| name_base(slider, label))
            .w_full()
            .h_7()
            .child(
                SliderTrack::new(state)
                    .relative()
                    .w_full()
                    .h_full()
                    .child(
                        div()
                            .absolute()
                            .top(px(13.))
                            .left_0()
                            .w_full()
                            .h(px(2.))
                            .bg(cx.theme().border),
                    )
                    .child(
                        SliderIndicator::new(state)
                            .absolute()
                            .top(px(13.))
                            .left_0()
                            .w_full()
                            .h(px(2.))
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .bottom_0()
                                    .left_0()
                                    .right(relative(1. - percentage))
                                    .bg(cx.theme().primary),
                            ),
                    )
                    .child(
                        SliderThumb::new(state)
                            .absolute()
                            .top(px(7.))
                            .left(relative(percentage))
                            .ml(px(-7.))
                            .size(px(14.))
                            .rounded(cx.theme().radius_full())
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().primary),
                    ),
            )
    }

    fn column(&self, base: bool, cx: &Context<Self>) -> impl IntoElement {
        let states = if base { &self.base } else { &self.component };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .gap_4()
            .p_4()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_lg)
            .child(div().font_semibold().child(if base {
                "Base Slider"
            } else {
                "Component Slider"
            }))
            .children(states.iter().enumerate().map(|(index, state)| {
                let name = ["Corner radius", "Padding scale"][index];
                let label = self.labels.map(|labels| labels[index]);
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div().flex().justify_between().text_sm().child(name).child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} px", state.read(cx).value().end())),
                        ),
                    )
                    .child(if base {
                        self.base_slider(state, label, cx).into_any_element()
                    } else {
                        Slider::new(state)
                            .map(|slider| name_component(slider, label))
                            .into_any_element()
                    })
            }))
    }

    fn observe(&self, window: &Window, cx: &Context<Self>) -> Vec<Observation> {
        assert_eq!(window.find_all_by_role(Role::Slider).len(), 4);
        [(&self.base, "Base"), (&self.component, "Component")]
            .into_iter()
            .flat_map(|(states, layer)| {
                states.iter().enumerate().map(move |(index, state)| {
                    let node = window.find(("slider", state.entity_id()));
                    assert_eq!(node.role(), Some(Role::Slider));
                    let bounds = node.bounds();
                    Observation {
                        layer,
                        intended_name: ["Corner radius", "Padding scale"][index],
                        role: format!("{:?}", node.role().unwrap()),
                        label: node.label().map(str::to_owned),
                        path: format!("{:?}", node.path()),
                        state_value: state.read(cx).value().end(),
                        bounds: [
                            bounds.origin.x.into(),
                            bounds.origin.y.into(),
                            bounds.size.width.into(),
                            bounds.size.height.into(),
                        ],
                    }
                })
            })
            .collect()
    }
}

impl Render for Review {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_6()
            .gap_6()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .text_sm()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .child("Slider accessibility / #3468"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child("Same values and presentation in both builds"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_6()
                    .child(self.column(true, cx))
                    .child(self.column(false, cx)),
            )
            .when_some(self.observations.as_ref(), |this, observations| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .pt_4()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .child(div().font_semibold().child("Observed Slider nodes"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Verification panel: labels read from rendered GPUI nodes"),
                        )
                        .children(observations.iter().map(|node| {
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(format!("{} / {}", node.layer, node.intended_name))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!(
                                                    "Role: {} | State value: {}",
                                                    node.role, node.state_value
                                                )),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(if node.label.is_some() {
                                            cx.theme().foreground
                                        } else {
                                            cx.theme().danger
                                        })
                                        .child(SharedString::from(format!(
                                            "label = {:?}",
                                            node.label
                                        ))),
                                )
                        })),
                )
            })
    }
}

fn main() {
    let output = std::env::args().nth(1).expect("output directory");
    let phase = std::env::args().nth(2).expect("before or after");
    let output = Path::new(&output);
    let mut results = Vec::new();
    let mut valid = true;

    for (mode, mode_name) in [(ThemeMode::Light, "light"), (ThemeMode::Dark, "dark")] {
        let mut cx = HeadlessAppContext::with_platform(
            gpui_kit::platform::current_platform(true).text_system(),
            Arc::new(Assets),
            gpui_kit::platform::current_headless_renderer,
        );
        cx.update(|cx| {
            gpui_kit::init(cx);
            Theme::change(mode, None, cx);
        });
        let (handle, view) = cx.update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds {
                        origin: Default::default(),
                        size: size(px(800.), px(720.)),
                    })),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
                cx,
                |_, cx| {
                    cx.new(|cx| Review {
                        base: [8., 16.].map(|value| {
                            cx.new(|_| SliderState::new().max(32.).step(1.).default_value(value))
                        }),
                        component: [8., 16.].map(|value| {
                            cx.new(|_| SliderState::new().max(32.).step(1.).default_value(value))
                        }),
                        labels: Some(["Corner radius", "Padding scale"]),
                        observations: None,
                    })
                },
            )
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| window.render_frame(cx))
            .unwrap();
        cx.capture_screenshot(handle)
            .expect("real offscreen renderer is required")
            .save(output.join(format!("{phase}-{mode_name}-controls.png")))
            .unwrap();

        let mut cases = Vec::new();
        for (case, labels) in [
            ("named", Some(["Corner radius", "Padding scale"])),
            ("renamed", Some(["圆角", "内边距"])),
            ("removed", None),
        ] {
            let observed = cx
                .update_window(handle, |_, window, cx| {
                    view.update(cx, |view, cx| {
                        view.labels = labels;
                        cx.notify();
                    });
                    window.render_frame(cx);
                    view.update(cx, |view, cx| view.observe(window, cx))
                })
                .unwrap();
            let passed = observed.iter().enumerate().all(|(index, node)| {
                node.label.as_deref() == labels.map(|labels| labels[index % 2])
                    && node.state_value == [8., 16.][index % 2]
            });
            valid &= passed;
            println!(
                "{phase} {mode_name} {case}: {}",
                if passed { "PASS" } else { "FAIL" }
            );
            cases.push(json!({
                "case": case,
                "expected_labels": labels,
                "passed": passed,
                "nodes": observed.iter().map(Observation::json).collect::<Vec<_>>(),
            }));
            if case == "named" {
                cx.update_window(handle, |_, window, cx| {
                    view.update(cx, |view, cx| {
                        view.observations = Some(observed);
                        cx.notify();
                    });
                    window.render_frame(cx);
                })
                .unwrap();
                cx.capture_screenshot(handle)
                    .expect("real offscreen renderer is required")
                    .save(output.join(format!("{phase}-{mode_name}.png")))
                    .unwrap();
            }
        }
        results.push(json!({"theme": mode_name, "cases": cases}));
    }
    std::fs::write(
        output.join(format!("{phase}.json")),
        serde_json::to_vec_pretty(&json!({
            "phase": phase,
            "builder_api_enabled": cfg!(feature = "fixed"),
            "renderer": "GPUI current_headless_renderer (Linux WGPU)",
            "metadata_source": "GPUI write_a11y_info observed on existing Slider IDs",
            "all_name_contracts_passed": valid,
            "results": results,
        }))
        .unwrap(),
    )
    .unwrap();
    if !valid {
        std::process::exit(1);
    }
}
