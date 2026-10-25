# Issue 3626 验证证据

本修复将文本选择注册与帧末发布分离，避免无关 GPUI Fast 视图反复重绘。基线为 `7b14e03f2ed4b46fc4149a8a523c7f91bcad6560`，代码提交为 `9b42fb299a50610d83c5fb484cf4db390871ac28`。证据保存在 fork 独立分支，不进入代码 PR。

- [测试结果与全量失败对照](3626-test-results.md)
- [性能方法、原始采样与前后 GPU 截图](performance/index.md)
- [逐段审查与真实桌面窗口验收截图](native-review/index.md)
- [English test results](3626-test-results.en.md) · [English performance report](performance/index.en.md) · [English review](native-review/index.en.md)

164 个 TextView 场景中，CPU 输入、绘制与后续任务处理耗时中位数为修复前 51.94 ms、修复后 2.58 ms；每次输入步骤的无关视图重绘中位数由 324 降至 0。这是 Linux dev 构建的 CPU 数据，不是屏幕呈现帧耗时或 FPS。每版每种数量 3 轮、共 180 个样本；一次输入步骤可能产生多次绘制。

性能目录内的前后截图为真实 GPU 离屏图，字节一致。native-review 中另有 Spectacle 捕获的实际桌面应用窗口。实体触摸屏输入未验证：触摸选择 UI 通过示例 Long press 按钮和真实指针拖柄操作验收，EIS 触摸长按未产生可确认的选区。首次 XWayland 滚轮操作未观察到滚动；随后在原生 Wayland 窗口确认页面移动且复制内容不变。

完整 Base Fast 测试修复前 35 项失败、修复后 23 项失败；修复后的 23 个失败名称全部也出现在基线，没有新增失败名称，不宣称全量通过。未运行 macOS 或 Windows。本轮审查没有追加生产代码修改。

## 重现有限规模回归

在包含此证据和修复提交的 fork 克隆中运行：

```sh
git fetch origin fix/3626-text-selection-frame-publication
python3 docs/evidence/text-selection-3626/reproduce.py --output /tmp/gpui-3626-review
# 如需真实字体和 GPU 性能测量与截图，使用新的输出目录：
python3 docs/evidence/text-selection-3626/reproduce.py --output /tmp/gpui-3626-measured --measure
```

脚本为两版创建独立 detached worktree，基线仅移植公共保留视图回归，生产实现不变；确认修复前回归失败、修复后通过，可选运行归档的性能夹具。每版使用独立构建目录，避免 Cargo 产物混用。worktree 和构建输出保留供检查；measure 模式需要原生构建依赖和真实离屏 GPU 渲染器。

发布时仅检查了重现脚本语法，没有重新执行完整构建。归档日志、性能采样与桌面验收结果是本次验证证据。报告中的绝对路径记录原测量环境，不是可移植重现脚本的前提；原生验收夹具按实际使用的源码保存，包含原本的本地输出路径。

AI 参与补丁分析、代码审查、回归与性能验证、原生窗口验收、中英文证据和 PR 准备。生产修复和新增回归来自用户提供的补丁，仍需维护者人工审阅。
