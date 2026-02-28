# eframe 桌面节点编辑器示例

这是一个基于 `eframe` / `egui` 的 Rust 桌面应用示例，当前重点是桌面端交互教学：

- 节点绘制与拖拽
- 端口连接与连线删除
- 可编辑标题和正文
- 画布平移、网格背景、缩放快捷键

项目定位是“学习型模板”，代码里保留了较多中文注释，便于初学者按函数理解状态管理和渲染流程。

## 环境要求

- Rust stable（建议使用 `rustup`）
- macOS / Windows / Linux 任一桌面系统

## 本地运行

```bash
cargo run --release
```

## 常用开发命令

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo check --workspace --all-targets --all-features
```

## 主要快捷键

- `Command + +` 或 `Command + =`：放大界面
- `Command + -`：缩小界面
- `Command + 0`：恢复 100% 缩放

在 Windows/Linux 上，`Command` 对应 `Ctrl`。

## 项目结构

- `src/app.rs`：核心界面逻辑（节点、连线、输入事件、绘制）
- `src/main.rs`：应用入口与窗口配置
- `src/lib.rs`：模块导出
- `assets/`：应用图标资源
- `.github/workflows/rust.yml`：基础 CI（fmt/clippy/test/check）
- `.github/workflows/build-desktop.yml`：自动构建 Windows/macOS 可执行文件

## 自动构建说明

仓库已配置 GitHub Actions 自动构建：

1. `CI` 工作流
   - 触发：`push main`、`pull_request`、手动触发
   - 内容：格式检查、Clippy、测试、编译检查

2. `Build Desktop Binaries` 工作流
   - 触发：`push main`、推送 `v*` 标签、手动触发
   - 产物：
     - `eframe_template-windows-x64`
     - `eframe_template-macos-intel`
     - `eframe_template-macos-apple-silicon`

构建完成后可在 GitHub Actions 的对应运行记录里下载 artifacts。

## Linux 依赖（如需）

Ubuntu/Debian:

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```
