# Issue 3626 Text Selection Fix Test Results

25 October 2026 · Linux x86_64 · rustc 1.98.0

**The target regression is reproduced before the patch and passes after it.** Four pointer moves increase an unrelated view's render count from 4 to 8 before the fix. The same test passes after the fix with no additional unrelated renders. Targeted selection tests pass. The complete Base suite still has 23 failures, all also present before the fix.

## Scope

Source issue [longbridge/gpui-kit 3626](https://github.com/longbridge/gpui-kit/issues/3626). Baseline `7b14e03f2ed4b46fc4149a8a523c7f91bcad6560`. Branch `fix/3626-text-selection-frame-publication`. The uncommitted patch modifies four Base files with 683 added and 235 removed lines.

Baseline worktree `/home/luren/Code/rust/gpui-kit-3626-before` contains only transplanted test helpers and one regression test; production code remains at the baseline. The fixed worktree is `/home/luren/Code/rust/gpui-component` and matches the supplied patch. Tests use real TextView retained views and mouse events in headless windows.

## Results

| Check | Result | Details | Evidence |
| --- | --- | --- | --- |
| Before drag regression | Expected failure | Unrelated view renders increase from 4 to 8 | [Log](logs/before-fast.log) |
| After GPUI Fast selection tests | 56 passed 0 failed | Includes four new regression cases | [Log](logs/after-fast.log) |
| After regular GPUI selection tests | 53 passed 0 failed | gpui-pre backend | [Log](logs/after-pre.log) |
| After Kit selection integration tests | 78 passed 0 failed | integration target filtered by selection | [Log](logs/kit-integration-fast.log) |
| Before all Base tests | 1454 passed 35 failed | Includes one transplanted regression case | [Log](logs/before-base-fast-all.log) |
| After all Base tests | 1469 passed 23 failed | All 23 failures also occur before the fix | [Log](logs/after-base-fast-all.log) |
| Formatting and patch checks | Passed | cargo fmt --all --check and git diff --check | [Log](logs/fmt.log) |

## New regression coverage

- Unrelated text views remain retained during four active-selection pointer moves. This fails before the patch and passes after it.
- Frame publication visits between N and 3N snapshots for N equal to 4 and 16. This measures operation counts, not frame latency.
- Scrolling actually changes the selected text offset without rebuilding unrelated document order.
- Selection and copy order survive partial frames, insertion and reordering. Removing and remounting an endpoint does not resurrect the retired gesture.

## Full suite comparison

There are zero new failing test names, 23 shared failures and 12 before-fix failures absent after the patch. These are observations from this run; the causes of the existing failures were not investigated.

### 23 shared failures

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

### 12 failures absent after the fix

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

## Rerun

```bash
# Fixed branch
cargo test -p gpui-base --features gpui-fast,test-support --lib text_selection --locked
cargo test -p gpui-base --features test-support --lib text_selection --locked
cargo test -p gpui-kit --features gpui-fast,test-support --test integration selection --locked
cargo test -p gpui-base --features gpui-fast,test-support --lib --locked
cargo fmt --all --check
git diff --check

# Saved before and after test binaries
/home/luren/Code/rust/gpui-component/target/3626/run-comparison.sh

# Manual validation in the respective worktree
cargo run -p text_selection --features gpui-fast --locked
```

The saved comparison script expects the baseline regression to fail, expects the fixed regression to pass, and runs the fixed selection suite. Local binaries are retained in target/3626. Use separate CARGO_TARGET_DIR values when rebuilding the two worktrees to avoid reuse of mismatched test artifacts.

## Performance and screenshots

[Measured comparison and real GPU offscreen screenshots](performance/index.en.md).

## Limits

Real GPU offscreen screenshots and CPU pointer-step measurements are now available. Native desktop interaction acceptance is now documented in the review report. Physical touchscreen input, cross-platform runs and display presentation latency remain unverified. The macOS timings in the issue belong to its reporter and were not reproduced here. Automated behavior and operation-count checks do not replace native-window acceptance.

Logs are included under logs and the failure comparison is in [comparison.json](comparison.json). The empty formatting log indicates a successful command with no output. An initial --test ui selection invocation matched zero tests; the reported 78 tests come from --test integration selection.

## Patch review and native acceptance

[Read the code review and native acceptance status](native-review/index.en.md)
