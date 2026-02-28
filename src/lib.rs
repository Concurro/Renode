#![warn(clippy::all, rust_2018_idioms)]

// 把 app 模块拆出来，避免 main.rs 过大，便于学习时分层阅读。
pub mod app;

// 重新导出 `TemplateApp`，这样外部可以直接用 `crate::TemplateApp` 访问。
