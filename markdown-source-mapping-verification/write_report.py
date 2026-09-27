#!/usr/bin/env python3
"""Render the measured results into the report in the project root."""
from pathlib import Path
import json

folder = Path(__file__).resolve().parent
rows = json.loads((folder / 'results.json').read_text())
complete = len(rows) == 23
ms = lambda value: f'{value / 1_000_000:,.3f}'
main = [row for row in rows if row['kind'] == 'alternating_spaces']
table = '\n'.join(
    f"| {row['raw_bytes'] // 1024} | {ms(row['old_align_ns'])} | {ms(row['new_align_ns'])} | "
    f"{ms(row['old_parse_ns'])} | {ms(row['new_parse_ns'])} | "
    f"{row['old_parse_ns'] / row['new_parse_ns']:.1f} 倍 |"
    for row in main
)
labels = {
    'horizontal_whitespace': '连续水平空白',
    'multiline_code': '多行代码',
    'unmapped_characters': '无法映射的字符',
    'entity_after_gaps': '映射缺口后的实体候选',
}
other_rows = [row for row in rows if row['kind'] in labels and row['raw_bytes'] in (65536, 65535)]
other_table = '\n'.join(
    f"| {labels[row['kind']]} | {row['raw_bytes']:,} | {ms(row['old_align_ns'])} | "
    f"{ms(row['new_align_ns'])} | {row['old_align_ns'] / row['new_align_ns']:.1f} 倍 |"
    for row in other_rows
)
parser_rows = [row for row in rows if row['kind'] in ('horizontal_whitespace', 'multiline_code')]
parser_table = '\n'.join(
    f"| {labels[row['kind']]} | {row['raw_bytes'] // 1024} | {row['source_bytes']:,} | "
    f"{ms(row['old_parse_ns'])} | {ms(row['new_parse_ns'])} | "
    f"{row['old_parse_ns'] / row['new_parse_ns']:.1f} 倍 |"
    for row in parser_rows if row['raw_bytes'] in (65536, 262144)
)
status = '全部 23 组性能用例已完成。' if complete else f'性能验证进行中：已完成 {len(rows)}/23 组，下方为已完成的实测结果。'
production = folder.parent / 'crates/base/src/text/format/markdown.rs'
module_status = (
    '临时验证模块在最终源码中已移除；保留的源码改动为补丁和 rustfmt 换行调整。'
    if not production.exists() or 'mod source_mapping_validation' not in production.read_text()
    else '临时验证模块目前用于执行性能测试，完成后会从产品源码移除。'
)
report = f'''# Markdown 源码映射修复验证

日期：2026-09-27。{status}

结论：补丁通过真实 Rust 编译、Markdown 回归测试、完整 `gpui-base` 库测试和 Clippy。重复空格输入的源码映射耗时已从平方增长降到接近线性增长；普通文本的返回向量也不再保留逐字符容量。

基线提交：`452e72f24a183174551fdec3db570997b1e1fc8f`。应用的补丁为用户提供的 `markdown-source-mapping.patch`；产品源码只修改 `crates/base/src/text/format/markdown.rs`，包含实现和五项新增回归测试。补丁文件的三处换行经过 rustfmt 调整，没有修改公开 API、Markdown 大小限制或 TextView 调度。

## 测量方法

- 环境：Linux x86_64，AMD Ryzen 9 7950X，rustc/cargo 1.98.0。
- 使用真实 `gpui-base` 测试二进制。旧 parser 从应用补丁前的源码逐字节提取，移除原有测试模块后纳入临时测试模块；新 parser 直接调用修改后的生产函数。旧、新版本使用相同依赖和配置。
- 性能测量设置 `profile.dev.package.gpui-base.opt-level=3`，其余依赖沿用仓库 dev 配置。这不是完整 release 构建。
- 每项预热一次，再测三次，表中取中位数。计时不包括输入构造、结果校验和返回结果的销毁；helper 内部的搜索、解码、段生成和旧版最终压缩均包含在内。
- “源码映射”只测 `aligned_source_segments`；“完整解析”测真实 `format::markdown::parse`，包含 mdast 解析及文档构造，无需调用选区 API。这些耗时不代表 GUI 帧延迟。
- 每组性能用例同时断言旧、新 helper 的映射段完全相同；有完整解析的用例还比较旧、新 `ParsedDocument`。

## 重复空格段落：修复前后

输入为 `"a ".repeat(n / 2)`，不包含换行。Markdown parser 会去掉最后一个尾随空格，因此 rendered 长度比表中的输入字节数少 1。

| 输入（KiB） | 修复前源码映射（ms） | 修复后源码映射（ms） | 修复前完整解析（ms） | 修复后完整解析（ms） | 完整解析加速 |
| ---: | ---: | ---: | ---: | ---: | ---: |
{table}

32 → 64 → 128 KiB 时，修复前源码映射约为 203 → 811 → 3,236 ms，每次翻倍输入约使时间增加四倍；修复后约为 0.276 → 0.551 → 1.136 ms，接近两倍增长。

## 其他触发形态

| 用例 | raw 字节数 | 修复前源码映射（ms） | 修复后源码映射（ms） | 加速 |
| --- | ---: | ---: | ---: | ---: |
{other_table}

连续水平空白为 `a + 空格串 + b`；多行代码 helper 的 raw 为重复 `x\\n`，完整 parser 输入额外包含代码围栏。无法映射字符用例和实体候选用例直接构造 helper 输入，用于覆盖失败搜索和缓存路径，不宣称这些 synthetic 输入会作为单个 mdast 文本节点自然出现。

| 完整解析用例 | 代码正文或文本（KiB） | Markdown 总字节数 | 修复前（ms） | 修复后（ms） | 加速 |
| --- | ---: | ---: | ---: | ---: | ---: |
{parser_table}

多行代码的完整解析仍呈现明显的超线性增长。表中的 helper 耗时已接近线性，但完整解析还包括其他处理；本次没有进一步定位或修改这些剩余成本。

23 组性能用例覆盖：重复空格 32/64/96/128/256 KiB，连续水平空白 8/16/32/64 KiB，多行代码 8/16/32/64/128/256 KiB，以及无法映射字符、实体候选各四种规模。各用例的实际 raw/rendered/source 字节数和三次样本保存在 `results.json`。

## 向量容量

96 KiB 重复空格段落，旧、新 helper 均返回一个映射段，但 `Vec::capacity()` 从 **98,303** 降为 **4**。本机每个 `SourceSegment` 为 32 字节，对应返回向量的元素存储从 **3,145,696 字节（约 3 MiB）** 降到 **128 字节**。

旧版先为每个字符生成一个段，再用原段数为压缩结果申请容量；在线合并同时消除了逐字符中间向量和过大的返回容量。这是实际返回容量测量，并非进程 RSS 或峰值堆内存测量。

## 正确性与静态检查

| 检查 | 结果 |
| --- | --- |
| 原版 Markdown 测试 | 52 passed，0 failed |
| 补丁版 Markdown 测试 | 57 passed，0 failed；五项新增回归测试均通过 |
| 补丁版完整 `gpui-base` 库测试 | 1,157 passed，0 failed |
| 实际 Rust helper 语义差分 | 2,018,720 组通过，覆盖 Unicode、空白、CR/LF、转义、有效及无效实体、缺口和非零源码偏移 |
| 公开 TextView 后台解析路径 | 通过；约 96 KiB 段落及代码块初始化、后台提交和 `set_text` 替换均完成，无选区调用 |
| 性能用例 | {'23 组通过' if complete else f'{len(rows)}/23 组已完成'}，每组同时校验旧、新映射结果 |
| `cargo clippy --locked -p gpui-base --all-targets -- -D warnings` | 通过 |
| 修改文件 rustfmt | 通过 |
| `cargo fmt -p gpui-base -- --check` | 通过 |
| `cargo fmt --all -- --check` | 未通过：未修改的 `crates/component/src/form/tests.rs` 存在既有格式差异；详见日志 |
| `git diff --check` | 通过 |

公开路径验证在 `gpui::TestAppContext` 中运行，使用真实 TextView 状态和测试 executor。没有测量原生窗口绘制、真实 executor 多文档饥饿或跨平台行为。常见 helper 路径为 O(N+M)，缺口索引路径上界为 O((N+M) log(N+1))；结果不表示整个 Markdown parser 对所有输入均为线性，也不构成任意大小文档的固定资源预算。

## 证据与复测

最终报告位于项目根目录；原始日志、源码快照及复测脚本位于项目内的 `markdown-source-mapping-verification/`。

- [修复前 Markdown 测试](markdown-source-mapping-verification/logs/baseline-markdown-tests.log)
- [修复后 Markdown 测试](markdown-source-mapping-verification/logs/patched-markdown-tests.log)
- [完整库测试](markdown-source-mapping-verification/logs/patched-base-tests.log)
- [Rust 差分与公开路径验证](markdown-source-mapping-verification/logs/rust-validation.log)
- [性能原始输出](markdown-source-mapping-verification/logs/benchmark.log)
- [中断前已完成的性能样本](markdown-source-mapping-verification/logs/benchmark-interrupted.log)
- [结构化性能结果](markdown-source-mapping-verification/results.json)
- [Clippy](markdown-source-mapping-verification/logs/clippy.log) · [Base 格式检查](markdown-source-mapping-verification/logs/fmt-base.log) · [最终全仓格式检查](markdown-source-mapping-verification/logs/fmt-final.log)
- [最终应用并格式化的补丁](markdown-source-mapping-verification/applied-and-formatted.patch) · [diff 检查](markdown-source-mapping-verification/logs/diff-check.log)
- [临时验证源码](markdown-source-mapping-verification/benchmark.rs) · [环境](markdown-source-mapping-verification/environment.json) · [输入与验证代码 SHA-256](markdown-source-mapping-verification/sha256.json)

性能运行曾为迁移输出目录而主动中止一次；已完成的四组结果保留，其余用例在迁移后继续执行。

在仓库根目录复测：

```sh
cargo test --locked -p gpui-base --lib text::format::markdown::tests
cargo test --locked -p gpui-base --lib
cargo fmt -p gpui-base -- --check
cargo clippy --locked -p gpui-base --all-targets -- -D warnings
git diff --check

# 临时注入验证模块，完成后自动恢复产品文件。
python3 markdown-source-mapping-verification/rerun.py
# 每次复测使用独立输出目录，不覆盖归档结果。
python3 markdown-source-mapping-verification/rerun.py --benchmark
```

{module_status}

每次复测的新日志和性能结果位于项目内 `target/markdown-source-mapping-reproduction/run-*/`。修复提交及交付记录见验证目录的 `PROVENANCE.json`。
'''
(folder.parent / 'markdown-source-mapping-verification.md').write_text(report)
print(folder.parent / 'markdown-source-mapping-verification.md')
