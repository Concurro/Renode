use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Key, PointerButton, Pos2, Rect, Sense, Stroke,
    StrokeKind, Vec2, epaint::CubicBezierShape,
};

// ============================================================
// 这份文件的目标：实现一个最小可用的“节点编辑器”界面
// 功能包括：
// 1) 渲染节点（标题栏 + 主体 + 输入/输出端口）
// 2) 节点之间显示连线（贝塞尔曲线）
// 3) 鼠标拖拽节点
// 4) 鼠标在空白处拖拽画布（平移视图）
// 5) 从输出端口拖到输入端口，创建一条新连线
//
// 代码阅读建议（初学者友好顺序）：
// 常量 -> 数据结构 -> 几何辅助函数 -> 绘制函数 -> 输入处理 -> update 主循环
// ============================================================

// 统一的节点尺寸，方便全局样式保持一致。
const NODE_SIZE: Vec2 = Vec2::new(180.0, 130.0);
// 标题栏高度。
const HEADER_HEIGHT: f32 = 28.0;
// 端口视觉半径（你看到的小圆点大小）。
// 端口命中半径（用于鼠标交互，通常比视觉半径大，便于点击/拖拽）。
const PORT_HIT_RADIUS: f32 = 10.0;
const NODE_INNER_PADDING_X: f32 = 10.0;
const NODE_INNER_PADDING_Y: f32 = 8.0;
const NODE_BG_COLOR: Color32 = Color32::from_rgb(30, 30, 35);
const NODE_BORDER_IDLE_COLOR: Color32 = Color32::from_rgb(82, 82, 91);
const NODE_BORDER_HOVER_COLOR: Color32 = Color32::from_rgb(148, 163, 184);
const NODE_HEADER_COLOR: Color32 = Color32::from_rgb(57, 116, 245);
const CANVAS_BG_COLOR: Color32 = Color32::from_rgb(20, 23, 29);
const SIDE_PANEL_BG: Color32 = Color32::from_rgb(25, 28, 34);
const LINK_COLOR: Color32 = Color32::from_rgb(122, 134, 156);
const DRAG_LINK_COLOR: Color32 = Color32::from_rgb(100, 180, 255);
const PORT_INPUT_COLOR: Color32 = Color32::from_rgb(255, 95, 87); // mac red
const PORT_OUTPUT_COLOR: Color32 = Color32::from_rgb(254, 188, 46); // mac yellow
const PORT_RADIUS: f32 = 6.5;
const PORT_RING_STROKE: f32 = 2.0;
const PORT_OUTSET: f32 = 8.0;
const ZOOM_STEP: f32 = 1.10;
const MIN_ZOOM_FACTOR: f32 = 0.60;
const MAX_ZOOM_FACTOR: f32 = 2.50;

/// 端口类型：输入端口 / 输出端口。
///
/// 在本示例中：
/// - 连线起点必须是 Output
/// - 连线终点必须是 Input
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PortKind {
    Input,
    Output,
}

/// 图中的一个节点。
#[derive(Clone, Debug)]
struct Node {
    /// 节点唯一 ID（逻辑标识，不是数组下标）。
    id: usize,
    /// 显示在标题栏的名称。
    title: String,
    /// 节点左上角在“世界坐标”中的位置。
    content: String, // 新增：可编辑正文
    ///
    /// 为什么要用世界坐标？
    /// - 画布可以平移（pan）
    /// - 屏幕坐标 = 世界坐标 + pan_offset
    position: Pos2,
    /// 节点尺寸。
    size: Vec2,
}

/// 正在拖拽“临时连线”时的状态。
#[derive(Clone, Copy, Debug)]
struct DragLinkState {
    /// 起始节点 ID。
    from_node: usize,
    /// 起始端口类型（本例中固定为 Output，但保留字段更易扩展）。
    from_port: PortKind,
    /// 鼠标当前屏幕坐标，用于实时绘制“跟手”的临时曲线。
    current_pos: Pos2,
}

/// 一条正式连线（保存到状态里）。
#[derive(Clone, Copy, Debug)]
struct Connection {
    /// 起点节点 ID（默认取该节点 Output 端口位置）。
    from_node_id: usize,
    /// 终点节点 ID（默认取该节点 Input 端口位置）。
    to_node_id: usize,
}

/// 整个节点编辑器 App 的运行时状态。
pub struct NodeGraphApp {
    /// 所有节点。
    nodes: Vec<Node>,
    /// 所有正式连线。
    connections: Vec<Connection>,
    /// 画布平移偏移量（世界坐标 -> 屏幕坐标）。
    pan_offset: Vec2,
    /// 当前是否处于“拖拽画布”模式。
    dragging_canvas: bool,
    /// 当前是否处于“拖拽连线”模式。
    dragging_link: Option<DragLinkState>,
    /// 下一次添加节点时使用的 ID（自增）。
    next_node_id: usize,
}

impl Default for NodeGraphApp {
    fn default() -> Self {
        // 初始化 3 个演示节点。
        let nodes = vec![
            Node {
                id: 0,
                title: "Input".to_owned(),
                content: "这里是节点说明".to_owned(),
                position: Pos2::new(100.0, 100.0),
                size: NODE_SIZE,
            },
            Node {
                id: 1,
                title: "Deal".to_owned(),
                content: "这里是节点说明".to_owned(),
                position: Pos2::new(340.0, 140.0),
                size: NODE_SIZE,
            },
            Node {
                id: 2,
                title: "Output".to_owned(),
                content: "这里是节点说明".to_owned(),
                position: Pos2::new(580.0, 100.0),
                size: NODE_SIZE,
            },
        ];

        Self {
            nodes,
            // 初始化两条演示连线：0 -> 1 -> 2
            connections: vec![
                Connection {
                    from_node_id: 0,
                    to_node_id: 1,
                },
                Connection {
                    from_node_id: 1,
                    to_node_id: 2,
                },
            ],
            pan_offset: Vec2::ZERO,
            dragging_canvas: false,
            dragging_link: None,
            next_node_id: 3,
        }
    }
}

impl NodeGraphApp {
    // ========================
    // 状态管理 / 数据查询
    // ========================

    /// 添加一个新节点。
    fn add_node(&mut self) {
        let id = self.next_node_id;
        self.next_node_id += 1;

        self.nodes.push(Node {
            id,
            title: format!("Node {id}"),
            // 简单错开位置，避免新节点完全重叠。
            position: Pos2::new(220.0 + (id as f32 * 24.0), 220.0),
            content: ".....".to_owned(),
            size: NODE_SIZE,
        });
    }

    /// 按节点 ID 查询节点引用。
    ///
    /// 注意：因为节点是 Vec 存储，ID 不一定等于下标，所以不要直接 `nodes[id]`。
    fn node_by_id(&self, id: usize) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    // ========================
    // 坐标与几何辅助
    // ========================

    /// 计算节点在“屏幕坐标”里的矩形。
    ///
    /// 核心公式：screen = world + pan_offset
    fn node_rect_screen(&self, node: &Node) -> Rect {
        Rect::from_min_size(node.position + self.pan_offset, node.size)
    }

    /// 计算某节点某端口在屏幕上的位置。
    /// - Input 在左边中点
    /// - Output 在右边中点
    fn port_pos_screen(&self, node: &Node, port: PortKind) -> Pos2 {
        let rect = self.node_rect_screen(node);
        match port {
            PortKind::Input => Pos2::new(rect.left() - PORT_OUTSET, rect.center().y),
            PortKind::Output => Pos2::new(rect.right() + PORT_OUTSET, rect.center().y),
        }
    }

    /// 命中测试：给定鼠标点，判断是否落在某个端口附近。
    ///
    /// 返回 `(node_id, port_kind)`，找不到则返回 `None`。
    fn port_at(&self, pointer_pos: Pos2) -> Option<(usize, PortKind)> {
        self.nodes.iter().find_map(|node| {
            let input = self.port_pos_screen(node, PortKind::Input);
            if input.distance(pointer_pos) <= PORT_HIT_RADIUS {
                return Some((node.id, PortKind::Input));
            }

            let output = self.port_pos_screen(node, PortKind::Output);
            if output.distance(pointer_pos) <= PORT_HIT_RADIUS {
                return Some((node.id, PortKind::Output));
            }

            None
        })
    }

    /// 判断鼠标是否在任意节点本体上（用于区分是拖节点还是拖画布）。
    fn is_pointer_over_node(&self, pointer_pos: Pos2) -> bool {
        self.nodes
            .iter()
            .any(|node| self.node_rect_screen(node).contains(pointer_pos))
    }

    fn cubic_bezier_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        Pos2::new(
            uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
            uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
        )
    }
    fn point_to_segment_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        let ap = p - a;
        let ab_len2 = ab.length_sq();
        if ab_len2 <= f32::EPSILON {
            return a.distance(p);
        }
        let t = (ap.dot(ab) / ab_len2).clamp(0.0, 1.0);
        let proj = a + t * ab;
        proj.distance(p)
    }

    fn hit_test_connection(&self, pointer: Pos2, threshold: f32) -> Option<usize> {
        self.connections.iter().enumerate().find_map(|(idx, conn)| {
            let from_node = self.node_by_id(conn.from_node_id)?;
            let to_node = self.node_by_id(conn.to_node_id)?;

            let from = self.port_pos_screen(from_node, PortKind::Output);
            let to = self.port_pos_screen(to_node, PortKind::Input);

            let horizontal = (to.x - from.x).abs();
            let curvature = horizontal.max(60.0) * 0.45;
            let c1 = from + Vec2::new(curvature, 0.0);
            let c2 = to - Vec2::new(curvature, 0.0);

            let mut min_d = f32::MAX;
            let samples = 24;
            let mut prev = from;
            for i in 1..=samples {
                let t = i as f32 / samples as f32;
                let cur = Self::cubic_bezier_point(from, c1, c2, to, t);
                min_d = min_d.min(Self::point_to_segment_distance(pointer, prev, cur));
                prev = cur;
            }

            (min_d <= threshold).then_some(idx)
        })
    }

    // ========================
    // 绘制相关
    // ========================

    /// 绘制一条贝塞尔曲线，用作连接线。
    ///
    /// 做法：
    /// - 起点：`from`
    /// - 终点：`to`
    /// - 两个控制点在水平方向展开，形成“流程图常见弯曲”
    fn draw_bezier(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
        let horizontal = (to.x - from.x).abs();
        let curvature = horizontal.max(60.0) * 0.45;

        let control_1 = from + Vec2::new(curvature, 0.0);
        let control_2 = to - Vec2::new(curvature, 0.0);

        painter.add(CubicBezierShape::from_points_stroke(
            [from, control_1, control_2, to],
            false,
            Color32::TRANSPARENT,
            Stroke::new(2.0, color),
        ));
    }

    /// 绘制所有“正式连线”。
    fn draw_connections(&self, ui: &mut egui::Ui) {
        let painter = ui.painter();

        for connection in &self.connections {
            let Some(from_node) = self.node_by_id(connection.from_node_id) else {
                // 节点可能被删除（未来扩展场景），找不到就跳过。
                continue;
            };
            let Some(to_node) = self.node_by_id(connection.to_node_id) else {
                continue;
            };

            let from = self.port_pos_screen(from_node, PortKind::Output);
            let to = self.port_pos_screen(to_node, PortKind::Input);
            Self::draw_bezier(painter, from, to, LINK_COLOR);
        }
    }

    /// 绘制“正在拖拽中的临时连线”。
    ///
    /// 当用户从输出端口按下并拖动时，这条线会跟随鼠标移动。
    fn draw_dragging_link(&self, ui: &mut egui::Ui) {
        let Some(link) = self.dragging_link else {
            return;
        };

        let Some(node) = self.node_by_id(link.from_node) else {
            return;
        };

        let from = self.port_pos_screen(node, link.from_port);
        Self::draw_bezier(ui.painter(), from, link.current_pos, DRAG_LINK_COLOR);
    }

    /// 绘制单个节点，并处理该节点相关输入（拖拽、端口交互）。
    fn draw_node(&mut self, ui: &mut egui::Ui, node_index: usize) {
        let node = &mut self.nodes[node_index];
        let node_rect = Rect::from_min_size(node.position + self.pan_offset, node.size);
        let header_rect =
            Rect::from_min_size(node_rect.min, Vec2::new(node_rect.width(), HEADER_HEIGHT));

        // 节点拖拽只放在标题栏，避免正文编辑区被拖拽逻辑抢事件。
        let drag_response = ui
            .allocate_rect(header_rect, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::Grab);
        if drag_response.dragged_by(PointerButton::Primary) {
            node.position += drag_response.drag_motion();
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        let node_id = node.id;
        let input_pos = Pos2::new(node_rect.left(), node_rect.center().y);
        let output_pos = Pos2::new(node_rect.right(), node_rect.center().y);
        let node_hovered = drag_response.hovered();

        // 端口命中区域（比视觉圆点大，增强可操作性）。
        let input_hit_rect = Rect::from_center_size(input_pos, Vec2::splat(PORT_HIT_RADIUS * 2.0));
        let output_hit_rect =
            Rect::from_center_size(output_pos, Vec2::splat(PORT_HIT_RADIUS * 2.0));

        // 给输入端口分配交互。
        let input_response = ui
            .interact(
                input_hit_rect,
                ui.make_persistent_id(("input_port", node_id)),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(CursorIcon::PointingHand);

        // 给输出端口分配交互。
        let output_response = ui
            .interact(
                output_hit_rect,
                ui.make_persistent_id(("output_port", node_id)),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(CursorIcon::PointingHand);

        // 当从输出端口开始拖拽时，进入“拖拽连线”状态。
        if output_response.drag_started() {
            let pointer_pos = output_response.interact_pointer_pos().unwrap_or(output_pos);

            self.dragging_link = Some(DragLinkState {
                from_node: node_id,
                from_port: PortKind::Output,
                current_pos: pointer_pos,
            });
        }

        // ---- 节点外观绘制 ----
        let border_color = if node_hovered {
            NODE_BORDER_HOVER_COLOR
        } else {
            NODE_BORDER_IDLE_COLOR
        };

        // 阴影层。
        ui.painter().rect_filled(
            node_rect.translate(Vec2::new(0.0, 3.0)).expand(1.0),
            CornerRadius::same(9),
            Color32::from_rgba_unmultiplied(0, 0, 0, 60),
        );

        // 节点主体背景。
        ui.painter()
            .rect_filled(node_rect, CornerRadius::same(8), NODE_BG_COLOR);
        // 节点边框。
        ui.painter().rect_stroke(
            node_rect,
            CornerRadius::same(8),
            Stroke::new(1.5, border_color),
            StrokeKind::Outside,
        );
        ui.painter().rect_filled(
            header_rect,
            CornerRadius {
                nw: 8,
                ne: 8,
                sw: 0,
                se: 0,
            },
            NODE_HEADER_COLOR,
        );

        // 文本框必须直接绑定 node 字段，才能真正修改状态。
        let title_rect = header_rect.shrink2(Vec2::new(NODE_INNER_PADDING_X, 5.0));
        let title_resp = ui.put(
            title_rect,
            egui::TextEdit::singleline(&mut node.title)
                .frame(false)
                .font(FontId::proportional(14.0))
                .text_color(Color32::WHITE)
                .desired_width(f32::INFINITY),
        );

        let content_rect = Rect::from_min_max(
            Pos2::new(
                node_rect.left() + NODE_INNER_PADDING_X,
                header_rect.bottom() + NODE_INNER_PADDING_Y,
            ),
            Pos2::new(
                node_rect.right() - NODE_INNER_PADDING_X,
                node_rect.bottom() - NODE_INNER_PADDING_Y,
            ),
        );
        // 正文编辑区背景：自绘浅色底，避免默认输入框黑底突兀。
        ui.painter()
            .rect_filled(content_rect, CornerRadius::same(6), NODE_BG_COLOR);
        let content_text_rect = content_rect.shrink2(Vec2::new(8.0, 6.0));
        let content_resp = ui.put(
            content_text_rect,
            egui::TextEdit::multiline(&mut node.content)
                .frame(false)
                .desired_width(content_text_rect.width())
                .desired_rows(Self::max_content_lines(node_rect))
                .font(FontId::proportional(12.0))
                .text_color(Color32::from_gray(220)),
        );
        Self::clamp_text_lines(&mut node.content, Self::max_content_lines(node_rect));

        // 输入/输出端口可视化：使用“插槽”风格而不是简单圆点。
        Self::draw_port_socket(ui, input_pos, PortKind::Input, input_response.hovered());
        Self::draw_port_socket(ui, output_pos, PortKind::Output, output_response.hovered());

        // 读取焦点状态，确保这些响应变量不是“仅创建未使用”。
        let _is_editing = title_resp.has_focus() || content_resp.has_focus();
    }

    /// 限制正文最多显示行数，防止在固定高度输入框里“回车无限下沉”。
    fn clamp_text_lines(text: &mut String, max_lines: usize) {
        let mut merged = String::new();
        for (idx, line) in text.split('\n').take(max_lines).enumerate() {
            if idx > 0 {
                merged.push('\n');
            }
            merged.push_str(line);
        }
        if *text != merged {
            *text = merged;
        }
    }

    fn max_content_lines(node_rect: Rect) -> usize {
        let content_height = node_rect.height() - HEADER_HEIGHT - NODE_INNER_PADDING_Y * 2.0 - 12.0;
        ((content_height / 18.0).floor() as usize).max(1)
    }

    /// 绘制端口：输入为空心环，输出为带实心核的圆点。
    /// 这是更常见的节点编辑器视觉语义。
    fn draw_port_socket(ui: &egui::Ui, center: Pos2, kind: PortKind, hovered: bool) {
        let color = match kind {
            PortKind::Input => PORT_INPUT_COLOR,
            PortKind::Output => PORT_OUTPUT_COLOR,
        };

        if hovered {
            ui.painter().circle_filled(
                center,
                PORT_RADIUS + 4.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 45),
            );
        }

        // 外环统一：与节点背景分离，辨识度更高。
        ui.painter()
            .circle_filled(center, PORT_RADIUS, NODE_BG_COLOR);
        ui.painter()
            .circle_stroke(center, PORT_RADIUS, Stroke::new(PORT_RING_STROKE, color));

        // 输入端口做“空心”语义；输出端口做“实心核”语义。
        match kind {
            PortKind::Input => {
                ui.painter().circle_filled(center, 2.0, NODE_BG_COLOR);
            }
            PortKind::Output => {
                ui.painter().circle_filled(
                    center,
                    2.6,
                    Color32::from_rgb(
                        color.r().saturating_sub(10),
                        color.g().saturating_sub(10),
                        color.b().saturating_sub(10),
                    ),
                );
            }
        }

        // 细外描边，提升在深色背景下的清晰度。
        ui.painter().circle_stroke(
            center,
            PORT_RADIUS + 0.5,
            Stroke::new(1.0, Color32::from_black_alpha(80)),
        );
    }

    fn draw_canvas_grid(ui: &egui::Ui, rect: Rect, pan_offset: Vec2) {
        let spacing_minor = 24.0;
        let spacing_major = spacing_minor * 4.0;
        let painter = ui.painter();
        let grid_minor_color = Color32::from_rgba_unmultiplied(120, 130, 150, 16);
        let grid_major_color = Color32::from_rgba_unmultiplied(120, 130, 150, 30);

        let offset_x_minor = pan_offset.x.rem_euclid(spacing_minor);
        let offset_y_minor = pan_offset.y.rem_euclid(spacing_minor);
        let offset_x_major = pan_offset.x.rem_euclid(spacing_major);
        let offset_y_major = pan_offset.y.rem_euclid(spacing_major);

        let mut x = rect.left() + offset_x_minor;
        while x <= rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, grid_minor_color),
            );
            x += spacing_minor;
        }

        let mut y = rect.top() + offset_y_minor;
        while y <= rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, grid_minor_color),
            );
            y += spacing_minor;
        }

        let mut x_major = rect.left() + offset_x_major;
        while x_major <= rect.right() {
            painter.line_segment(
                [
                    Pos2::new(x_major, rect.top()),
                    Pos2::new(x_major, rect.bottom()),
                ],
                Stroke::new(1.0, grid_major_color),
            );
            x_major += spacing_major;
        }

        let mut y_major = rect.top() + offset_y_major;
        while y_major <= rect.bottom() {
            painter.line_segment(
                [
                    Pos2::new(rect.left(), y_major),
                    Pos2::new(rect.right(), y_major),
                ],
                Stroke::new(1.0, grid_major_color),
            );
            y_major += spacing_major;
        }
    }

    fn handle_zoom_shortcuts(ctx: &egui::Context) {
        let mut zoom_in = false;
        let mut zoom_out = false;
        let mut zoom_reset = false;

        ctx.input(|i| {
            if i.modifiers.command {
                zoom_in = i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals);
                zoom_out = i.key_pressed(Key::Minus);
                zoom_reset = i.key_pressed(Key::Num0);
            }
        });

        if zoom_reset {
            ctx.set_zoom_factor(1.0);
            ctx.request_repaint();
            return;
        }

        let current = ctx.zoom_factor();
        let next = if zoom_in {
            Some((current * ZOOM_STEP).clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR))
        } else if zoom_out {
            Some((current / ZOOM_STEP).clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR))
        } else {
            None
        };

        if let Some(zoom_factor) = next {
            ctx.set_zoom_factor(zoom_factor);
            ctx.request_repaint();
        }
    }
    // ========================
    // 输入收尾处理
    // ========================

    /// 在鼠标松开时，尝试结束“拖拽连线”。
    ///
    /// 规则：
    /// 1) 只有拖到 Input 端口才创建连线
    /// 2) 不允许自己连自己
    /// 3) 不允许重复连线
    fn finish_dragging_link_if_needed(&mut self, ctx: &egui::Context) {
        let Some(link) = self.dragging_link else {
            return;
        };

        // 只在“鼠标左键已松开”时结算。
        if !ctx.input(|i| i.pointer.primary_down()) {
            if let Some(pointer_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                if let Some((target_node_id, target_port)) = self.port_at(pointer_pos) {
                    let duplicate_exists = self.connections.iter().any(|connection| {
                        connection.from_node_id == link.from_node
                            && connection.to_node_id == target_node_id
                    });

                    if target_port == PortKind::Input
                        && target_node_id != link.from_node
                        && !duplicate_exists
                    {
                        self.connections.push(Connection {
                            from_node_id: link.from_node,
                            to_node_id: target_node_id,
                        });
                    }
                }
            }

            // 无论是否连接成功，都退出临时拖拽状态。
            self.dragging_link = None;
        }
    }

    /// 处理画布平移（Pan）。
    ///
    /// 关键思路：
    /// - 只有在“空白区域按下并拖动”才平移
    /// - 若起始点在节点或端口上，则不进入平移
    fn handle_canvas_pan(&mut self, canvas_response: &egui::Response, ctx: &egui::Context) {
        if canvas_response.drag_started_by(PointerButton::Primary) {
            self.dragging_canvas =
                canvas_response
                    .interact_pointer_pos()
                    .is_some_and(|pointer_pos| {
                        !self.is_pointer_over_node(pointer_pos)
                            && self.port_at(pointer_pos).is_none()
                    });
        }

        if self.dragging_canvas && canvas_response.dragged_by(PointerButton::Primary) {
            self.pan_offset += canvas_response.drag_motion();
            // 交互中主动请求重绘，保证拖拽流畅。
            ctx.request_repaint();
        }

        if canvas_response.drag_stopped_by(PointerButton::Primary)
            || !ctx.input(|i| i.pointer.primary_down())
        {
            self.dragging_canvas = false;
        }
    }
}

impl eframe::App for NodeGraphApp {
    /// 每一帧都会调用 `update`。
    ///
    /// 你可以把它理解为 UI 主循环：
    /// 1) 画侧边栏（按钮、信息）
    /// 2) 画中央画布（连接线、节点、临时线）
    /// 3) 更新交互状态（鼠标拖拽、松开结算）
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::handle_zoom_shortcuts(ctx);

        // ---------- 左侧控制面板 ----------
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(220.0)
            .frame(
                egui::Frame::new()
                    .fill(SIDE_PANEL_BG)
                    .inner_margin(egui::Margin::symmetric(12, 10)),
            )
            .show(ctx, |ui| {
                ui.heading("Node Control");
                ui.separator();

                if ui.button("Add Node").clicked() {
                    self.add_node();
                }

                if ui.button("Reset View").clicked() {
                    self.pan_offset = Vec2::ZERO;
                }

                if ui.button("Clear Links").clicked() {
                    self.connections.clear();
                }

                ui.separator();
                ui.label(format!("Nodes: {}", self.nodes.len()));
                ui.label(format!("Links: {}", self.connections.len()));
            });

        // ---------- 中央画布 ----------
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(CANVAS_BG_COLOR))
            .show(ctx, |ui| {
                // 给整个中央区域注册一个可拖拽响应，专门用于“画布平移”。
                let canvas_rect = ui.max_rect();
                Self::draw_canvas_grid(ui, canvas_rect, self.pan_offset);
                let canvas_response = ui.allocate_rect(canvas_rect, Sense::drag());

                // 绘制顺序很重要：
                // 先画连接线（在下层）
                // 再画节点（在上层）
                self.draw_connections(ui);
                self.draw_dragging_link(ui);

                for node_index in 0..self.nodes.len() {
                    self.draw_node(ui, node_index);
                }

                // 如果正在拖拽临时连线，每帧更新鼠标位置。
                if let Some(link) = &mut self.dragging_link {
                    if let Some(pointer_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        link.current_pos = pointer_pos;
                        ctx.request_repaint();
                    }
                }
                if ctx.input(|i| i.pointer.button_clicked(PointerButton::Secondary)) {
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if let Some(index) = self.hit_test_connection(pos, 10.0) {
                            self.connections.remove(index);
                        }
                    }
                }
                // 先结算“连线拖拽是否结束”，再处理“画布平移”。
                self.finish_dragging_link_if_needed(ctx);
                self.handle_canvas_pan(&canvas_response, ctx);
            });
    }
}
