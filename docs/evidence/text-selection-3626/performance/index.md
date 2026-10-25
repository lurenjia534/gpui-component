# Issue 3626 修复前后性能与截图

164 个 TextView 场景中，每次拖动步骤的 CPU 耗时中位数从 51.94 ms 降至 2.58 ms，约提升 20.2 倍；无关视图重绘次数的中位数从 324 降至 0。

2026-10-25 · Linux x86_64 · rustc 1.98.0 · GPUI Fast 0.1.10

## 性能对照

![CPU time and unrelated render comparison](performance-comparison.png)

| TextView 数量 | 修复前耗时中位数 | 修复后耗时中位数 | 速度比 | 无关视图重绘中位数 |
| --- | --- | --- | --- | --- |
| 16 | 2.19 ms | 0.50 ms | 4.4× | 28 → 0 |
| 64 | 11.83 ms | 1.16 ms | 10.2× | 124 → 0 |
| 164 | 51.94 ms | 2.58 ms | 20.2× | 324 → 0 |

## 修复前后截图

同一场景 64 个 TextView 相同拖选终点 点击查看原图

修复前

![修复前](before.png)

修复后

![修复后](after.png)

## 测量方法

同一份测试程序、GPUI Fast、浅色主题和固定 1180 × 1400 逻辑像素窗口，文本按 4 列排列。只拖选第一个 TextView，其他文本不变。每个数量跑 3 轮，每轮预热 10 次、记录 60 次，总计每版每种数量 180 个样本。鼠标位置沿相同路径往返变化，每一步调用真实鼠标事件、window.draw 和 run_until_parked。记录的时间从输入派发前到绘制和后续任务处理完成，计数来自每个文本所属视图的 render 调用。没有强制刷新整个窗口。

## 范围和解读

这是 Linux x86_64 本机 dev 构建的 CPU 工作量对照。GPUI 依赖沿用仓库的 opt-level 3 配置，应用及框架其余部分为 dev 配置。一个输入步骤可能导致多次绘制，所以“每步重绘次数”不等于“每帧参与者数量”。计时不含截图读取、GPU 提交完成或显示器呈现，也不等同于 Issue 报告者的 macOS 帧耗时；不得据此换算 FPS。截图使用真实平台字体和 GPUI GPU 离屏渲染，非模拟界面，亦非桌面截屏。两版截图字节完全一致，说明本场景的选区外观保持一致；性能差异由计数和耗时展示。

## 原始证据

[before.csv](before.csv) · [after.csv](after.csv) · [summary.json](summary.json) · [before-run.log](before-run.log) · [after-run.log](after-run.log) · [selection_3626.rs](selection_3626.rs)

本机重跑命令

```bash
/home/luren/Code/rust/gpui-component/target/3626/before-evidence before /home/luren/Desktop/3626-test-results/performance
/home/luren/Code/rust/gpui-component/target/3626/after-evidence after /home/luren/Desktop/3626-test-results/performance
```

[新增真实桌面交互验收与系统窗口截图](../native-review/index.md)
