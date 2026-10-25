# Issue 3626 补丁代码审查与原生验收记录

2026-10-25 · fix/3626-text-selection-frame-publication

## 目的与方案

本补丁解决 GPUI Fast 文本选择过程中无关视图重复重绘，以及参与者重复发布造成的二次复杂度。改动涉及 Base 的 4 个文件。将几何注册与全量快照发布分离、保留跨帧顺序的方案与该问题相符，且保留现有组件导入和调用方式。

## 审查结论

结论：逐段审查后，尚未确认需要阻止本 PR 的代码缺陷。审查覆盖原补丁全部改动及邻接调用路径，没有对生产实现追加修改。该结论仍受下方原生交互验收状态限制。

| 范围 | 审查内容 |
| --- | --- |
| 注册与发布 | TextSelectionHandle::register 通过只读窗口实体记录几何，只计算自身快照；schedule_selection_frame 每帧只排队一次全量发布，避免 N 次注册各扫描 N 个参与者。 |
| 缓存与文档顺序 | ParticipantPaintOrder 保留旧顺序；完整帧提交顺序，局部帧仅在新增、几何变化或逆序时请求一次完整恢复。外部 with_document_order 仍使用调用者提供的顺序。 |
| 卸载与生命周期 | 弱实体、渲染标记及 generation 区分缓存和离场；端点离场清除共享手势，已移除参与者清理本地快照，避免保留实体重新挂载后恢复旧选区。 |
| 通知和复制缓存 | set_snapshot_quietly 使本地投影缓存即时失效；draw_changed 合并帧内变化；帧末收集更新后才逐个通知，避免扫描时嵌套写窗口实体。 |
| 触摸和滚动 | 触摸边缘变化及保持中的光标刷新延后处理；自身滚动以 scroll_offset 更新投影，文档顺序不因内容滚动而重建。 |
| 公共 API 与层次 | 改动限于 Base 行为；未增加或移除公开导出。GlobalState::begin_selection_frame 的公开签名保留为兼容空操作；新增 paint_order、状态类型和方法均非新增公开入口。 |
| 测试审查 | 4 项新增测试使用实际保留 TextView、鼠标事件及复制查询，分别覆盖无关重绘、线性访问、滚动及顺序和卸载。snapshot_visits 仅在 cfg(test) 存在。 |

## 验证与限制

- cargo clippy -p gpui-base --features gpui-fast,test-support --lib --locked -- --deny warnings 通过，日志为 [clippy.log](clippy.log)。
- 原补丁差异检查通过；生产工作区仍为同一 4 个修改文件。
- 已有行为验证：Fast 选择 56 项、常规 GPUI 选择 53 项、Kit 相关集成 78 项通过。
- 已有全量对照：修复后 1469 通过、23 失败；23 项全部在基线失败集中，没有新增失败名称。性能测量和离屏截图见 [性能对照](../performance/index.md)。

## 原生桌面验收状态

**原生桌面交互验收已完成，未发现新的阻塞问题。**测试使用 KDE Wayland 授权的 RemoteDesktop EIS 输入，在实际 XWayland 与 Wayland 应用窗口中操作。以下截图由 Spectacle 捕获实际桌面应用窗口，包含系统窗口边框，不是离屏渲染截图。

| 场景 | 结果 | 证据 |
| --- | --- | --- |
| 跨文本正向与反向拖选 | 通过；四段文本的复制内容逐字相同，保持文档顺序 | [正向复制](cross-forward-copy.txt) · [反向复制](cross-reverse-copy.txt) |
| 触摸选择菜单与拖柄 | 通过；示例 Long press 按钮产生选词与拖柄，真实鼠标拖动拖柄扩选，菜单 Copy 与 Ctrl+C 内容一致 | [拖柄扩选复制](touch-handle-copy.txt) · [菜单复制](touch-menu-copy.txt) |
| 滚动中的选区与拖柄 | 通过；Wayland 页面确实滚动，选区和拖柄跟随文本移动，滚动前后复制均为 ProseMirror | [滚动前复制](wayland-before-scroll-copy.txt) · [滚动后复制](wayland-after-scroll-copy.txt) |
| 端点卸载与保留实体重新挂载 | 通过；移除前有三段有效选区，移除后无选区，重新挂载后仍无选区 | [移除前状态](unmount-phase-0.txt) · [移除后状态](unmount-phase-1.txt) · [恢复后状态](unmount-phase-2.txt) |

端点用例由临时验收程序的键盘 Action 触发，避免先点击按钮清空选区。夹具将 Ctrl+C 临时绑定为“记录当前选区后依次移除、恢复、复查”，不是复制行为测试；实际复制验证在未修改的 text_selection 和 touch_selection 示例中完成。[夹具源码](selection_native_review.rs) · [构建日志](native-fixture-build.log) · [验收记录](acceptance.json)

## 真实桌面截图

### 正向跨文本选区

![正向跨文本选区](cross-forward.png)

### 反向跨文本选区

![反向跨文本选区](cross-reverse.png)

### 触摸选择菜单与拖柄

![触摸选择菜单与拖柄](touch-handles.png)

### 拖柄扩选

![拖柄扩选](touch-handle-drag.png)

### Wayland 滚动前选区

![Wayland 滚动前选区](wayland-handles.png)

### Wayland 滚动后选区与拖柄

![Wayland 滚动后选区与拖柄](wayland-selection-scrolled.png)

### 端点移除前

![端点移除前](unmount-selected.png)

### 端点已移除

![端点已移除](endpoint-removed.png)

### 端点恢复后旧选区没有复活

![端点恢复后旧选区没有复活](endpoint-restored.png)

## 验证边界

实体触摸屏的硬件输入链路未验收。EIS 触摸长按没有在目标窗口产生可确认的选区；本报告只将桌面示例 Long press 入口、菜单和拖柄的实际桌面操作记为通过，不将注入的触摸长按记为通过。首次 XWayland 滚轮操作未观察到页面移动；随后在原生 Wayland 窗口完成了有可见滚动证据的验证。未做 macOS、Windows 或实体触屏设备测试。上述未完成部分不应在 PR 中写成已通过。

结合逐段代码审查、Clippy、已有回归与集成测试，以及本轮原生桌面验收，目前可以提交 PR 供维护者审阅。应披露原有 23 项全量失败、平台范围、性能计时口径及 AI 参与；本轮未追加生产代码修改。
