# Markdown source mapping 验证素材

基线：`452e72f24a183174551fdec3db570997b1e1fc8f`。
`markdown-before.rs` 是应用补丁前的原始文件；`markdown-after.rs` 是应用补丁并执行 rustfmt 后的产品文件。

`benchmark.rs` 临时作为 `markdown.rs` 的测试子模块编译。旧 parser 由 `markdown-before.rs` 移除测试模块后逐字节提取到 `markdown-before-core.rs`，通过测试包装函数访问私有 helper；旧、新 parser 使用同一个 gpui-base 二进制、相同依赖和优化配置。

`results.json` 包含每项三次计时、中位数、字节数、段数和返回向量容量。`environment.json` 保存工具链、CPU 和优化配置；`sha256.json` 保存输入及验证代码的校验值。`PROVENANCE.json` 记录基线、修复提交及验证范围；`PROCESS.md` 记录本次实际执行过程。

在仓库根目录执行：

```sh
python3 markdown-source-mapping-verification/rerun.py
python3 markdown-source-mapping-verification/rerun.py --benchmark
```

脚本只接受与本次 `markdown-after.rs` 完全相同的源码，运行前临时添加验证模块，结束后恢复产品文件。如果运行期间文件被其他程序修改，脚本会拒绝覆盖该文件。每次的新日志及结果放入项目内 `target/markdown-source-mapping-reproduction/run-*/`，不覆盖归档结果。请勿在复测期间同时编辑产品文件。

一般检查命令：

```sh
cargo test --locked -p gpui-base --lib text::format::markdown::tests
cargo test --locked -p gpui-base --lib
cargo fmt -p gpui-base -- --check
cargo fmt --all -- --check
cargo clippy --locked -p gpui-base --all-targets -- -D warnings
git diff --check
```

全仓格式检查在未修改的 `crates/component/src/form/tests.rs` 中存在既有差异。最终状态以仓库根目录的 `markdown-source-mapping-verification.md` 为准。
