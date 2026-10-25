# Issue 3626 文本选择修复测试结果

2026年10月25日 · Linux x86_64 · rustc 1.98.0 · 自动化测试记录

**目标回归已复现并在补丁应用后通过。**拖动第一段文本的选区时，修复前无关视图累计 render 次数从 4 增至 8；修复后同一条用例确认没有新增重绘。相关选择测试全部通过，但 Base 全量仍有 23 个修复前也存在的失败项，不能表述为全量测试通过。

## 验证范围

问题来源为 [longbridge/gpui-kit Issue 3626](https://github.com/longbridge/gpui-kit/issues/3626)。基线提交为 `7b14e03f2ed4b46fc4149a8a523c7f91bcad6560`，修复分支为 `fix/3626-text-selection-frame-publication`。补丁只修改 Base 层 4 个文件，683 行新增、235 行删除，尚未提交。

修复前 worktree 位于 `/home/luren/Code/rust/gpui-kit-3626-before`。为进行同用例对照，该 worktree 仅移植了测试夹具和一条回归测试，生产实现保持基线版本。修复后 worktree 位于 `/home/luren/Code/rust/gpui-component`，生产改动与用户提供的补丁一致。测试使用真实 TextView、保留视图及鼠标事件，在无头测试窗口中验证。

## 测试结果

| 检查项 | 结果 | 说明 | 原始日志 |
| --- | --- | --- | --- |
| GPUI Fast 拖选回归对照 | 修复前失败 修复后通过 | 无关视图 render 从 4 增到 8；修复后计数保持不变 | [before-fast.log](logs/before-fast.log) · [replayed.log](logs/replayed.log) |
| 修复后 GPUI Fast 选择测试 | 56 通过 0 失败 | 包含 4 项新增回归测试 | [after-fast.log](logs/after-fast.log) |
| 修复后常规 GPUI 选择测试 | 53 通过 0 失败 | gpui-pre 后端 | [after-pre.log](logs/after-pre.log) |
| 修复后 Kit 选择集成测试 | 78 通过 0 失败 | integration 测试目标 按 selection 筛选 | [kit-integration-fast.log](logs/kit-integration-fast.log) |
| 修复前 Base 全量测试 | 1454 通过 35 失败 | 包含移植的 1 条回归用例 | [before-base-fast-all.log](logs/before-base-fast-all.log) |
| 修复后 Base 全量测试 | 1469 通过 23 失败 | 23 个失败项全部与修复前重合 | [after-base-fast-all.log](logs/after-base-fast-all.log) |
| 格式与补丁检查 | 通过 | cargo fmt --all --check 与 git diff --check | [fmt.log](logs/fmt.log) |

## 新增回归覆盖

- `active_selection_keeps_unrelated_text_views_retained`：四次选区移动后，无关视图不应重新 render；修复前断言失败，修复后通过。
- `active_selection_frame_publication_visits_participants_linearly`：分别使用 4 和 16 个参与者，快照访问次数位于 N 到 3N 之间；这是操作计数验证，不是帧耗时基准。
- `scrolling_selected_text_does_not_rebuild_unrelated_document_order`：确认内容确实滚动，且不导致无关视图全量重建。
- `retained_text_order_survives_partial_frames_insertion_and_reordering`：覆盖局部重绘、插入、重排、端点卸载和重新挂载后的选择与复制状态。

## 全量失败对照

比较两版失败用例名称后，新增失败为 0，重合失败为 23；修复前的 12 个失败在修复后未再出现。该结果描述本次运行，不代表已定位全部原有失败的根因。

### 修复前后均失败的 23 项

1. `dock::tab_group::tests::an_empty_group_asks_the_renderer_for_its_empty_state`
2. `dock::tab_group::tests::the_renderer_composes_frame_then_tab_bar_then_content`
3. `input::state::tests::test_input_does_not_invalidate_cached_parent_during_paint`
4. `input::state::tests::test_scroll_bounce_releases_when_the_end_packet_is_horizontal`
5. `input::state::tests::test_scroll_bounce_stretches_past_the_end_and_springs_back`
6. `nav_stack::tests::the_outgoing_view_is_dropped_once_its_exit_has_run`
7. `popover::tests::default_open_renders_content_without_activation`
8. `popup::tests::the_popup_surface_blocks_the_panel_it_covers`
9. `popup::tests::trigger_capture_enables_deferred_content_on_the_next_frame`
10. `scroll_bounce::tests::list_stretches_only_the_distance_past_either_edge`
11. `scroll_bounce::tests::phaseless_wheel_scrolls_back_right_after_bouncing`
12. `scroll_bounce::tests::trackpad_direction_change_between_frames_does_not_bounce_in_the_middle`
13. `scroll_bounce::tests::trackpad_release_between_frames_does_not_bounce_in_the_middle`
14. `scrollbar::tests::repeated_touch_and_mouse_drags_keep_the_painted_grab_point`
15. `text::state::tests::reveal_range::a_reveal_that_cannot_be_shown_gives_up`
16. `text::text_view::tests::long_press_release_keeps_handles_which_drag_the_selection`
17. `text::text_view::tests::long_press_selects_word_then_drag_extends_selection`
18. `text::text_view::tests::ordered_markdown_list_start_reaches_layout_marker`
19. `text::text_view::tests::table_row_backgrounds_follow_the_frame_corner_radius`
20. `toast::tests::bottom_stack_reflow_moves_up_without_an_opposite_direction_jump`
21. `toast::tests::keyed_stack_reflow_moves_from_the_current_visual_position`
22. `virtual_list::tests::horizontal_visible_range_and_deferred_scroll_are_preserved`
23. `virtual_list::tests::vertical_visible_range_and_deferred_scroll_are_preserved`

### 修复后未再失败的 12 项

1. `text::inline_virtual_tests::inline_virtual_resource_growth_remeasures_all_blocks_and_preserves_copy`
2. `text::state::tests::range_highlights::a_dragged_selection_highlights_through_its_source_range`
3. `text::text_view::tests::copy_before_paint::virtualized_flow_retains_its_source_owned_partial_anchor`
4. `text::text_view::tests::inline_object_inherits_bold_and_link_clicks_without_opening_on_drag`
5. `text::text_view::tests::inline_plugins_drag_and_copy_across_formulas_in_both_directions`
6. `text::text_view::tests::inline_resource_size_update_reflows_without_changing_selection`
7. `text::text_view::tests::scaled_inline_code_keeps_links_and_drag_selection`
8. `text::text_view::tests::selection_band_beyond_clip_copies_offscreen_text`
9. `text::text_view::tests::table_cell_copy_excludes_following_blocks_in_a_scrollable_view`
10. `text::text_view::tests::table_cell_selection_does_not_leak_into_another_window`
11. `text::text_view::tests::table_drag_from_a_cell_stays_in_that_cell`
12. `text_selection::tests::active_selection_keeps_unrelated_text_views_retained`

## 重跑方式

```bash
# 当前修复分支
cargo test -p gpui-base --features gpui-fast,test-support --lib text_selection --locked
cargo test -p gpui-base --features test-support --lib text_selection --locked
cargo test -p gpui-kit --features gpui-fast,test-support --test integration selection --locked
cargo test -p gpui-base --features gpui-fast,test-support --lib --locked
cargo fmt --all --check
git diff --check

# 已保存的测试二进制前后对照
/home/luren/Code/rust/gpui-component/target/3626/run-comparison.sh

# 手工验证入口 在对应 worktree 运行
cargo run -p text_selection --features gpui-fast --locked
```

前后对照脚本先确认修复前回归失败，再确认修复后同用例通过，最后执行修复后的 56 项选择测试。二进制位于本机 `target/3626`，运行脚本前无需重新编译。若要重新构建两版，建议为两个 worktree 分配独立的 `CARGO_TARGET_DIR`，避免切换源码时复用不对应的测试产物。

## 性能与截图补充

[查看修复前后性能曲线、数据与两版截图](performance/index.md)。新增同场景真实 GPU 离屏截图及 CPU 输入步骤测量。

## 验证边界

已生成真实 GPU 离屏窗口截图，真实桌面窗口交互已补充验证，见审查与原生验收记录；跨平台、实体触摸屏和屏幕呈现帧耗时测量尚未完成。Issue 中的 macOS 帧耗时属于报告者数据，本次未复测。此文档提供自动化行为验证、操作计数验证及失败集对照，不能替代原生窗口体验验收。

原始日志保存在随附的 `logs` 目录；[comparison.json](comparison.json) 保存完整失败集对照。空的格式日志表示命令无输出且成功退出。曾尝试以 `--test ui selection` 筛选，但该目标未匹配任何用例；此处 78 项结果来自正确的 `--test integration selection`，不计入空运行。

## 补丁审查与原生验收

[查看逐段审查结论及原生桌面验收状态](native-review/index.md)
