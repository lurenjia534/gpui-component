# 稀疏 Markdown 表格分配：修复前后验证

[English](README.md)

验证日期：2026-10-24。基线提交：
`69a2077c2da588de1c46b89ffd903e1373e45a29`。
在 `fix/sparse-markdown-table` 分支应用 `gpui-kit-sparse-table.patch`。
代码提交：`0e7a58062a94dcbea667d6f2962140fe1d8538de`。
环境：Linux、rustc 1.98.0、cargo 1.98.0。
本报告和验证工具由 AI 辅助整理。

## 验证结果

修复后，生产 Markdown 转换器只保留源文提供的单元格。
修复前，每个正文行都会补齐到表头列数，使紧凑的稀疏输入产生平方级的保留对象数量。
下表是**生产解析器输出中的实际 `TableCell` 数量**，不是只检查依赖 AST 的结果，也不是推算值。

| 表头列数 | 正文行数 | 输入字节数 | 修复前保留单元格 | 修复后保留单元格 | 消除的补齐单元格 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 64 | 452 | 2,080 | 96 | 1,984 |
| 64 | 128 | 900 | 8,256 | 192 | 8,064 |
| 128 | 256 | 1,796 | 32,896 | 384 | 32,512 |

原始输出：[修复前](before.log)、[修复后](after.log)。
两个维度同时翻倍时，修复前保留数量约增至四倍，修复后只增至两倍。

## 方法与复现

临时探针 [probe.rs](probe.rs) 直接调用生产函数
`text::format::markdown::parse`，然后统计返回文档中所有
`TableRow.children.len()` 的总和。输入构造如下：

```rust
format!(
    "{}|\n{}|\n{}",
    "|h".repeat(columns),
    "|---".repeat(columns),
    "|x|\n".repeat(rows),
)
```

修复前运行只将已应用补丁中的 `truncate` 恢复成原来的 `resize_with`，
其余生产代码相同；修复后使用 `truncate`。
两次运行使用同一份有界探针，没有尝试耗尽内存。
验证结束后已从生产源码移除临时探针，只留下补丁自带的回归测试。

在已应用补丁的仓库中执行，期间不要并发修改源码：

```sh
python3 docs/evidence/sparse-markdown-table/reproduce.py
```

脚本临时加入探针，并在 `finally` 中恢复源码。探针对应的命令为：

```sh
cargo test -p gpui-base --features test-support --offline --lib \
  text::format::markdown::sparse_table_evidence -- --exact --nocapture
```

当前仓库的现有测试需要 `test-support`。
首次未启用该 feature 的运行因 `a11y_disabled.rs` 引用了受 feature 控制的
`crate::test_support` 而编译失败，启用后已解决。
本次环境具备离线依赖，没有沿用此前报告中无法运行生产测试的限制。

## 回归验证

- 新增的稀疏单元格回归测试在**旧补齐逻辑下失败**：
  [before-regression.log](before-regression.log)，退出码 101。
  失败断言是“每个单单元格正文行只保留一个单元格”。
- 相同测试在**修复后通过**：
  [after-regression.log](after-regression.log)，退出码 0。
- 不等长行语义测试**通过**：
  [ragged-regression.log](ragged-regression.log)。
  覆盖短行、超出表头列数的截断，以及显式中间空单元格的保留；
  各行单元格数量为 `[3, 1, 3, 3]`。
- `cargo fmt --all --check` 和 `git diff --check` 通过。
  [format.log](format.log) 为空，表示格式检查成功。

Base 库与测试的 Clippy 检查也已通过：

```sh
cargo clippy --locked --offline -p gpui-base --features test-support --lib --tests -- -D warnings
```

原始输出：[clippy.log](clippy.log)。

## 修复原理

修复前：

```rust
row.children.resize_with(table.column_aligns.len(), Default::default);
table.children.push(row);
```

修复后：

```rust
row.children.truncate(table.column_aligns.len());
table.children.push(row);
```

对本次输入结构，输入字节数为 `6*C + 4*R + 4`。
旧逻辑保留 `C*(R+1)` 个单元格，其中 `R*(C-1)` 个是额外生成的；
新逻辑只保留 `C+R` 个。
默认 `TableCell` 持有默认 `Paragraph`，后者包含
`Arc<Mutex<InlineState>>`，因此补齐会创建并保留非平凡对象。

当 `C=R=10,000` 时，由公式可推算：100,004 字节输入对应修复前
100,010,000 个保留单元格，修复后 20,000 个。
**本次没有执行这个大规模案例**；这些数字来自代码与有界实测的推算，
不能当作内存分配量、峰值 RSS 或崩溃的实测结果。

## 可达路径与对齐检查

当前源码中的相关位置：

- `crates/base/src/text/text_view.rs`：`TextView::markdown` 接收文本，
  布局阶段构造 `TextViewState::markdown`。
- `crates/base/src/text/state.rs`：Markdown 路径的 `parse_content` 调用
  `format::markdown::parse`，追加文本需要重新解析的内容也经过该入口。
- `crates/base/src/text/format/markdown.rs`：`Node::Table` 从解析器的对齐列表
  构造 `column_aligns`，每行均进入 `parse_table_row`。
- `crates/base/src/text/node.rs`：`TableCell` 持有 `Paragraph`。
  `BlockNode::render_table_row` 遍历共享列，并通过
  `row.children.get(ix)` 读取可选内容。因此，缺失的尾部单元格不需要存储
  占位 `Paragraph`，也能占据对应列的位置。
- `crates/component/src/input/popovers/hover_popover.rs` 将悬停文档交给
  `popovers/mod.rs` 的 `render_markdown`，再进入 `TextView::markdown`。
  该路径需要已配置的 provider 和用户交互；provider 默认未设置。

只有宿主应用渲染攻击者可影响的 Markdown 时，才形成相应信任边界。
公开的渲染 API 不代表存在公开网络入口。
同步与后台解析的大小阈值只决定调度方式，不是分配预算。

## 验证边界

本次证据证明的是新增的**解析阶段保留单元格扩张**已被消除。
没有声称完成 GUI 截图、视觉对齐实测、完整悬停交互、峰值 RSS 测量、
故意 OOM 或完整测试套件。
对齐兼容性依据渲染源码检查和解析回归测试，尚无截图验证。

该补丁没有增加通用文档预算，也未解决上游解析器已有的 CPU 放大。
渲染器仍为每行遍历共享列网格并创建展示元素，因此本报告不证明恶意表格
在布局和渲染阶段具有有界成本。

## PR 描述建议

标题：`text: Preserve sparse Markdown table rows`

让 Markdown 表格缺失的单元格保持隐式状态，避免为每个缺失列分配默认
`TableCell`。继续按 GFM 表头列数截断多余单元格；渲染器已有共享列网格，
可处理缺失内容。

生产解析器有界实测表明，1,796 字节输入的保留单元格从 32,896 减至 384。
稀疏单元格回归测试在旧逻辑下失败、在修复后通过；不等长行语义测试和格式检查通过。
没有公共 API 变更。补丁验证与证据由 AI 辅助整理，仍需人工审核。
