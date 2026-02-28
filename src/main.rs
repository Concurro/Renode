#![warn(clippy::all, rust_2018_idioms)]
// 在 Windows 的 release 模式下隐藏控制台窗口（避免弹黑框）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe_template::app::NodeGraphApp;

// 程序入口：
// `eframe::Result` 是 eframe 约定的返回类型，便于统一处理启动错误。
fn main() -> eframe::Result {
    // 初始化日志系统。设置 `RUST_LOG=debug` 后可看到更多调试日志。
    env_logger::init();

    // NativeOptions = 桌面端窗口配置（大小、图标、渲染相关参数等）
    let native_options = eframe::NativeOptions {
        // Viewport 可以理解为“窗口外观/行为配置器”
        viewport: egui::ViewportBuilder::default()
            // 初始窗口大小（像素）
            .with_inner_size([400.0, 300.0])
            // 允许用户缩小到的最小尺寸（防止 UI 被压扁）
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // 设置窗口图标（可选）
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[0..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    // `run_native` 会开启事件循环（event loop）并持续刷新 UI。
    // 第三个参数是一个“应用创建函数”：
    // - `cc` 是 CreationContext（创建上下文），包含 egui 上下文和持久化存储等信息
    // - 返回我们自己的 App 状态对象 `TemplateApp`
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| {
            configure_system_font(&cc.egui_ctx);
            Ok(Box::new(NodeGraphApp::default()))
        }),
    )
}

fn configure_system_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 根据操作系统选择字体路径
    let font_path = if cfg!(target_os = "windows") {
        "C:\\Windows\\Fonts\\simhei.ttf"
    } else if cfg!(target_os = "macos") {
        "/System/Library/Fonts/Hiragino Sans GB.ttc"
    } else if cfg!(target_os = "linux") {
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"
    } else {
        panic!("不支持的操作系统");
    };

    // 读取字体文件（如果文件不存在则回退）
    match std::fs::read(font_path) {
        Ok(font_data) => {
            fonts.font_data.insert(
                "system_chinese".to_owned(),
                egui::FontData::from_owned(font_data.into()).into(),
            );

            // 将中文字体作为后备字体添加到 Proportional 家族
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("system_chinese".to_owned());
        }
        Err(e) => {
            eprintln!("警告：无法加载系统字体 '{}'：{}", font_path, e);
            // 可以 fallback 到内置字体（但无法显示中文）
            // 或者提示用户手动放置字体文件
        }
    }

    ctx.set_fonts(fonts);
}
