mod calculator;
mod graph;

use arboard::Clipboard;
use image::{ImageBuffer, Rgb};

use graph::{sample_derivative, sample_function, sample_parametric, GraphCanvas};
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, column, container, row, text, text_input, Row, Canvas};
use iced::mouse;
use iced::{application, Element, Fill, Font, Point, Subscription, Theme};

// Brutalist color palette
mod colors {
    use iced::Color;

    pub const BACKGROUND: Color = Color::from_rgb(0.1, 0.1, 0.1);
    pub const SURFACE: Color = Color::from_rgb(0.18, 0.18, 0.18);
    pub const BUTTON: Color = Color::from_rgb(0.22, 0.22, 0.22);
    pub const TEXT: Color = Color::WHITE;
    pub const TEXT_DIM: Color = Color::from_rgb(0.6, 0.6, 0.6);
    pub const ACCENT_ORANGE: Color = Color::from_rgb(1.0, 0.42, 0.21);
    pub const ACCENT_TEAL: Color = Color::from_rgb(0.31, 0.8, 0.77);
    pub const ACCENT_RED: Color = Color::from_rgb(0.9, 0.25, 0.25);
    pub const ERROR: Color = Color::from_rgb(1.0, 0.3, 0.3);
    pub const SUCCESS: Color = Color::from_rgb(0.3, 0.9, 0.4);
    pub const TAB_ACTIVE: Color = Color::from_rgb(0.31, 0.8, 0.77);
    pub const TAB_INACTIVE: Color = Color::from_rgb(0.22, 0.22, 0.22);
}

fn main() -> iced::Result {
    application("RustyAbacus", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window_size((500.0, 700.0))
        .run()
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum Mode {
    #[default]
    Calculator,
    Graph,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum GraphType {
    #[default]
    Function,
    Parametric,
}

struct App {
    mode: Mode,
    // Calculator state
    calc_input: String,
    calc_result: String,
    calc_error: Option<String>,
    // Memory
    memory: f64,
    // History
    history: Vec<(String, f64)>, // (expression, result)
    last_answer: Option<f64>,
    // Graph state
    graph_canvas: GraphCanvas,
    graph_expr: String,
    graph_expr_x: String, // For parametric x(t)
    graph_expr_y: String, // For parametric y(t)
    graph_type: GraphType,
    // Multiple curves
    additional_curves: Vec<String>, // Additional f(x) expressions
    t_range: (f64, f64),            // Range for parametric t
    show_derivative: bool,          // Show derivative of main function
    show_help: bool,                 // Show keyboard shortcuts help
    cursor_world: Option<(f64, f64)>,
    is_dragging: bool,
    last_drag_pos: Option<Point>,
}

impl Default for App {
    fn default() -> Self {
        let mut graph_canvas = GraphCanvas::new();
        graph_canvas.viewport.set_size(460.0, 350.0);

        Self {
            mode: Mode::Calculator,
            calc_input: String::new(),
            calc_result: String::new(),
            calc_error: None,
            memory: 0.0,
            history: Vec::new(),
            last_answer: None,
            graph_canvas,
            graph_expr: String::from("sin(x)"),
            graph_expr_x: String::from("cos(t)"),
            graph_expr_y: String::from("sin(t)"),
            graph_type: GraphType::Function,
            additional_curves: Vec::new(),
            t_range: (0.0, 2.0 * std::f64::consts::PI),
            show_derivative: false,
            show_help: false,
            cursor_world: None,
            is_dragging: false,
            last_drag_pos: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    // Mode switching
    SetMode(Mode),
    // Calculator messages
    CalcInput(char),
    CalcFunction(String),
    CalcClear,
    CalcBackspace,
    CalcEvaluate, // Press equals / save to history
    KeyPressed(Key, Modifiers),
    // Memory messages
    MemoryAdd,
    MemorySubtract,
    MemoryRecall,
    MemoryClear,
    // History
    UseHistoryItem(usize),
    InsertAns,
    // Graph messages
    GraphExprChanged(String),
    GraphExprXChanged(String),
    GraphExprYChanged(String),
    SetGraphType(GraphType),
    GraphResetView,
    // Multi-curve support
    AddCurve,
    RemoveCurve(usize),
    UpdateCurve(usize, String),
    // Parametric t-range
    SetTRangeStart(String),
    SetTRangeEnd(String),
    // Derivative
    ToggleDerivative,
    // Clipboard
    CopyResult,
    // Help
    ToggleHelp,
    // Export
    ExportGraph,
    // Canvas mouse events
    CanvasMouseDown,
    CanvasMouseUp,
    CanvasMouseMove(Point),
    CanvasScroll(mouse::ScrollDelta),
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)))
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SetMode(mode) => {
                self.mode = mode;
                if mode == Mode::Graph {
                    self.update_graph();
                }
            }
            Message::CalcInput(c) => {
                self.calc_input.push(c);
                self.evaluate_calc();
            }
            Message::CalcFunction(func) => {
                self.calc_input.push_str(&func);
                self.calc_input.push('(');
                self.evaluate_calc();
            }
            Message::CalcClear => {
                self.calc_input.clear();
                self.calc_result.clear();
                self.calc_error = None;
            }
            Message::CalcBackspace => {
                self.calc_input.pop();
                self.evaluate_calc();
            }
            Message::CalcEvaluate => {
                // Save current expression to history if valid
                if let Ok(value) = calculator::evaluate(&self.calc_input) {
                    self.history.push((self.calc_input.clone(), value));
                    self.last_answer = Some(value);
                    // Keep only last 20 history items
                    if self.history.len() > 20 {
                        self.history.remove(0);
                    }
                }
            }
            Message::MemoryAdd => {
                if let Ok(value) = self.calc_result.parse::<f64>() {
                    self.memory += value;
                }
            }
            Message::MemorySubtract => {
                if let Ok(value) = self.calc_result.parse::<f64>() {
                    self.memory -= value;
                }
            }
            Message::MemoryRecall => {
                self.calc_input = format!("{}", self.memory);
                self.evaluate_calc();
            }
            Message::MemoryClear => {
                self.memory = 0.0;
            }
            Message::UseHistoryItem(index) => {
                if let Some((expr, _)) = self.history.get(index) {
                    self.calc_input = expr.clone();
                    self.evaluate_calc();
                }
            }
            Message::InsertAns => {
                if let Some(ans) = self.last_answer {
                    self.calc_input.push_str(&format!("{}", ans));
                    self.evaluate_calc();
                }
            }
            Message::KeyPressed(key, _modifiers) => {
                if self.mode == Mode::Calculator {
                    self.handle_calc_key(key);
                } else {
                    self.handle_graph_key(key);
                }
            }
            Message::GraphExprChanged(expr) => {
                self.graph_expr = expr;
                self.update_graph();
            }
            Message::GraphExprXChanged(expr) => {
                self.graph_expr_x = expr;
                self.update_graph();
            }
            Message::GraphExprYChanged(expr) => {
                self.graph_expr_y = expr;
                self.update_graph();
            }
            Message::SetGraphType(graph_type) => {
                self.graph_type = graph_type;
                self.graph_canvas.is_parametric = graph_type == GraphType::Parametric;
                self.update_graph();
            }
            Message::GraphResetView => {
                self.graph_canvas.viewport.reset();
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Message::AddCurve => {
                if self.additional_curves.len() < 4 {
                    self.additional_curves.push(String::new());
                }
            }
            Message::RemoveCurve(index) => {
                if index < self.additional_curves.len() {
                    self.additional_curves.remove(index);
                    self.update_graph();
                }
            }
            Message::UpdateCurve(index, expr) => {
                if index < self.additional_curves.len() {
                    self.additional_curves[index] = expr;
                    self.update_graph();
                }
            }
            Message::SetTRangeStart(val) => {
                if let Ok(v) = val.parse::<f64>() {
                    self.t_range.0 = v;
                    self.update_graph();
                }
            }
            Message::SetTRangeEnd(val) => {
                if let Ok(v) = val.parse::<f64>() {
                    self.t_range.1 = v;
                    self.update_graph();
                }
            }
            Message::ToggleDerivative => {
                self.show_derivative = !self.show_derivative;
                self.update_graph();
            }
            Message::CopyResult => {
                if !self.calc_result.is_empty() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(&self.calc_result);
                    }
                }
            }
            Message::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Message::ExportGraph => {
                self.export_graph_to_image();
            }
            Message::CanvasMouseDown => {
                self.is_dragging = true;
                self.last_drag_pos = None;
            }
            Message::CanvasMouseUp => {
                self.is_dragging = false;
                self.last_drag_pos = None;
            }
            Message::CanvasMouseMove(position) => {
                let (wx, wy) = self.graph_canvas.viewport.screen_to_world(position.x, position.y);
                self.cursor_world = Some((wx, wy));
                self.graph_canvas.set_cursor(Some(position));

                if self.is_dragging {
                    if let Some(last_pos) = self.last_drag_pos {
                        let dx = position.x - last_pos.x;
                        let dy = position.y - last_pos.y;
                        self.graph_canvas.viewport.pan(-dx, -dy);
                        self.graph_canvas.clear_cache();
                        self.update_graph();
                    }
                    self.last_drag_pos = Some(position);
                }
            }
            Message::CanvasScroll(delta) => {
                let scroll_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
                };
                let factor = if scroll_y > 0.0 { 1.1 } else { 0.9 };
                // Zoom centered at cursor position if available, otherwise center of canvas
                if let Some(cursor) = self.graph_canvas.cursor_position {
                    self.graph_canvas.viewport.zoom(factor as f64, cursor.x, cursor.y);
                } else {
                    let center_x = self.graph_canvas.viewport.width / 2.0;
                    let center_y = self.graph_canvas.viewport.height / 2.0;
                    self.graph_canvas.viewport.zoom(factor as f64, center_x, center_y);
                }
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
        }
    }

    fn handle_calc_key(&mut self, key: Key) {
        match key.as_ref() {
            Key::Character("0") => self.calc_input.push('0'),
            Key::Character("1") => self.calc_input.push('1'),
            Key::Character("2") => self.calc_input.push('2'),
            Key::Character("3") => self.calc_input.push('3'),
            Key::Character("4") => self.calc_input.push('4'),
            Key::Character("5") => self.calc_input.push('5'),
            Key::Character("6") => self.calc_input.push('6'),
            Key::Character("7") => self.calc_input.push('7'),
            Key::Character("8") => self.calc_input.push('8'),
            Key::Character("9") => self.calc_input.push('9'),
            Key::Character("+") => self.calc_input.push('+'),
            Key::Character("-") => self.calc_input.push('-'),
            Key::Character("*") => self.calc_input.push('*'),
            Key::Character("/") => self.calc_input.push('/'),
            Key::Character("^") => self.calc_input.push('^'),
            Key::Character("(") => self.calc_input.push('('),
            Key::Character(")") => self.calc_input.push(')'),
            Key::Character(".") => self.calc_input.push('.'),
            Key::Character(c) if c.len() == 1 => {
                let ch = c.chars().next().unwrap();
                if ch.is_alphabetic() {
                    self.calc_input.push(ch.to_ascii_lowercase());
                }
            }
            Key::Named(keyboard::key::Named::Backspace) => {
                self.calc_input.pop();
            }
            Key::Named(keyboard::key::Named::Escape) => {
                self.calc_input.clear();
                self.calc_result.clear();
                self.calc_error = None;
                return;
            }
            Key::Named(keyboard::key::Named::Tab) => {
                self.mode = if self.mode == Mode::Calculator {
                    Mode::Graph
                } else {
                    Mode::Calculator
                };
                if self.mode == Mode::Graph {
                    self.update_graph();
                }
                return;
            }
            _ => return,
        }
        self.evaluate_calc();
    }

    fn handle_graph_key(&mut self, key: Key) {
        match key.as_ref() {
            Key::Character("+") | Key::Character("=") => {
                self.graph_canvas
                    .viewport
                    .zoom(1.2, self.graph_canvas.viewport.width / 2.0, self.graph_canvas.viewport.height / 2.0);
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Character("-") => {
                self.graph_canvas
                    .viewport
                    .zoom(0.8, self.graph_canvas.viewport.width / 2.0, self.graph_canvas.viewport.height / 2.0);
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Character("r") => {
                self.graph_canvas.viewport.reset();
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Named(keyboard::key::Named::ArrowUp) => {
                self.graph_canvas.viewport.pan(0.0, -20.0);
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Named(keyboard::key::Named::ArrowDown) => {
                self.graph_canvas.viewport.pan(0.0, 20.0);
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Named(keyboard::key::Named::ArrowLeft) => {
                self.graph_canvas.viewport.pan(-20.0, 0.0);
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Named(keyboard::key::Named::ArrowRight) => {
                self.graph_canvas.viewport.pan(20.0, 0.0);
                self.graph_canvas.clear_cache();
                self.update_graph();
            }
            Key::Named(keyboard::key::Named::Tab) => {
                self.mode = Mode::Calculator;
            }
            _ => {}
        }
    }

    fn evaluate_calc(&mut self) {
        if self.calc_input.is_empty() {
            self.calc_result.clear();
            self.calc_error = None;
            return;
        }

        match calculator::evaluate(&self.calc_input) {
            Ok(value) => {
                if value.fract() == 0.0 && value.abs() < 1e15 {
                    self.calc_result = format!("{}", value as i64);
                } else {
                    self.calc_result = format!("{:.10}", value)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string();
                }
                self.calc_error = None;
            }
            Err(e) => {
                self.calc_result.clear();
                self.calc_error = Some(e);
            }
        }
    }

    fn update_graph(&mut self) {
        let (x_min, x_max) = self.graph_canvas.viewport.x_range();
        let num_samples = 1000;

        match self.graph_type {
            GraphType::Function => {
                // Sample main curve
                let points = sample_function(&self.graph_expr, x_min, x_max, num_samples);
                self.graph_canvas.set_points(points);

                // Sample additional curves
                let additional: Vec<Vec<_>> = self.additional_curves
                    .iter()
                    .filter(|expr| !expr.is_empty())
                    .map(|expr| sample_function(expr, x_min, x_max, num_samples))
                    .collect();
                self.graph_canvas.set_additional_curves(additional);

                // Sample derivative if enabled
                if self.show_derivative {
                    let deriv_points = sample_derivative(&self.graph_expr, x_min, x_max, num_samples);
                    self.graph_canvas.set_derivative(deriv_points, true);
                } else {
                    self.graph_canvas.set_derivative(Vec::new(), false);
                }
            }
            GraphType::Parametric => {
                let points = sample_parametric(
                    &self.graph_expr_x,
                    &self.graph_expr_y,
                    self.t_range.0,
                    self.t_range.1,
                    num_samples,
                );
                self.graph_canvas.set_parametric_points(points);
                self.graph_canvas.set_additional_curves(Vec::new());
                self.graph_canvas.set_derivative(Vec::new(), false);
            }
        }
    }

    fn export_graph_to_image(&self) {
        let width = 800u32;
        let height = 600u32;

        // Create image buffer with dark background
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_, _| {
            Rgb([26u8, 26u8, 26u8]) // colors::BACKGROUND
        });

        let scale_x = width as f64 / self.graph_canvas.viewport.width as f64;
        let scale_y = height as f64 / self.graph_canvas.viewport.height as f64;

        // Get viewport ranges (for future grid drawing)
        let _ranges = (
            self.graph_canvas.viewport.x_range(),
            self.graph_canvas.viewport.y_range(),
        );

        // Draw curves
        let curve_colors: [Rgb<u8>; 5] = [
            Rgb([79, 205, 196]),   // Teal
            Rgb([242, 153, 51]),   // Orange
            Rgb([179, 102, 230]),  // Purple
            Rgb([51, 204, 102]),   // Green
            Rgb([230, 77, 128]),   // Pink
        ];

        // Helper to convert world coords to image coords
        let world_to_img = |wx: f64, wy: f64| -> (i32, i32) {
            let (sx, sy) = self.graph_canvas.viewport.world_to_screen(wx, wy);
            ((sx as f64 * scale_x) as i32, (sy as f64 * scale_y) as i32)
        };

        // Draw main curve
        let points = if self.graph_canvas.is_parametric {
            &self.graph_canvas.parametric_points
        } else {
            &self.graph_canvas.points
        };

        let color = if self.graph_canvas.is_parametric {
            Rgb([255, 107, 53]) // Orange for parametric
        } else {
            curve_colors[0]
        };

        for window in points.windows(2) {
            if window[0].valid && window[1].valid {
                let (x1, y1) = world_to_img(window[0].x, window[0].y);
                let (x2, y2) = world_to_img(window[1].x, window[1].y);
                draw_line(&mut img, x1, y1, x2, y2, color);
            }
        }

        // Draw additional curves
        for (i, curve) in self.graph_canvas.additional_curves.iter().enumerate() {
            let color = curve_colors[(i + 1) % 5];
            for window in curve.windows(2) {
                if window[0].valid && window[1].valid {
                    let (x1, y1) = world_to_img(window[0].x, window[0].y);
                    let (x2, y2) = world_to_img(window[1].x, window[1].y);
                    draw_line(&mut img, x1, y1, x2, y2, color);
                }
            }
        }

        // Draw axes
        let axis_color = Rgb([255u8, 255u8, 255u8]);
        let (origin_x, origin_y) = world_to_img(0.0, 0.0);
        // X axis
        if origin_y >= 0 && origin_y < height as i32 {
            for x in 0..width {
                if let Some(pixel) = img.get_pixel_mut_checked(x, origin_y as u32) {
                    *pixel = axis_color;
                }
            }
        }
        // Y axis
        if origin_x >= 0 && origin_x < width as i32 {
            for y in 0..height {
                if let Some(pixel) = img.get_pixel_mut_checked(origin_x as u32, y) {
                    *pixel = axis_color;
                }
            }
        }

        // Save file using file dialog
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .set_file_name("graph.png")
            .save_file()
        {
            if let Err(e) = img.save(&path) {
                eprintln!("Failed to save image: {}", e);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tab_bar = self.view_tabs();

        let content: Element<'_, Message> = if self.show_help {
            self.view_help()
        } else {
            match self.mode {
                Mode::Calculator => self.view_calculator(),
                Mode::Graph => self.view_graph(),
            }
        };

        let main_content = column![tab_bar, content].spacing(0);

        container(main_content)
            .width(Fill)
            .height(Fill)
            .style(|_theme| container::Style {
                background: Some(colors::BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }

    fn view_tabs(&self) -> Element<'_, Message> {
        let calc_tab = self.tab_button("CALC", Mode::Calculator);
        let graph_tab = self.tab_button("GRAPH", Mode::Graph);

        let help_btn = button(
            text("?").size(16).font(Font::MONOSPACE).color(colors::TEXT).center()
        )
        .width(35)
        .height(35)
        .style(|_theme, _status| button::Style {
            background: Some(colors::BUTTON.into()),
            text_color: colors::TEXT,
            border: iced::Border {
                color: colors::TEXT,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .on_press(Message::ToggleHelp);

        container(row![calc_tab, graph_tab, help_btn].spacing(2))
            .padding(10)
            .width(Fill)
            .into()
    }

    fn view_help(&self) -> Element<'_, Message> {
        let calc_shortcuts = column![
            text("CALCULATOR SHORTCUTS").size(16).font(Font::MONOSPACE).color(colors::ACCENT_TEAL),
            text("").size(8),
            text("0-9, +, -, *, /    Basic input").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("^                  Power").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("( )                Parentheses").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("Backspace          Delete last").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("Escape             Clear all").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("Tab                Switch mode").size(13).font(Font::MONOSPACE).color(colors::TEXT),
        ].spacing(4);

        let graph_shortcuts = column![
            text("GRAPH SHORTCUTS").size(16).font(Font::MONOSPACE).color(colors::ACCENT_ORANGE),
            text("").size(8),
            text("Mouse drag         Pan view").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("Scroll wheel       Zoom in/out").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("+/=                Zoom in").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("-                  Zoom out").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("Arrow keys         Pan view").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("R                  Reset view").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("Tab                Switch mode").size(13).font(Font::MONOSPACE).color(colors::TEXT),
        ].spacing(4);

        let functions = column![
            text("FUNCTIONS").size(16).font(Font::MONOSPACE).color(colors::SUCCESS),
            text("").size(8),
            text("sin, cos, tan      Trig functions").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("asin, acos, atan   Inverse trig").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("log, ln            Logarithms").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("sqrt, abs          Square root, absolute").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("floor, ceil        Rounding").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("n!                 Factorial").size(13).font(Font::MONOSPACE).color(colors::TEXT),
            text("pi, e              Constants").size(13).font(Font::MONOSPACE).color(colors::TEXT),
        ].spacing(4);

        let close_btn = button(
            text("Close").size(14).font(Font::MONOSPACE).color(colors::BACKGROUND)
        )
        .style(|_theme, _status| button::Style {
            background: Some(colors::ACCENT_TEAL.into()),
            text_color: colors::BACKGROUND,
            border: iced::Border {
                color: colors::TEXT,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding([8, 20])
        .on_press(Message::ToggleHelp);

        container(
            column![
                text("KEYBOARD SHORTCUTS").size(20).font(Font::MONOSPACE).color(colors::TEXT),
                text("").size(10),
                calc_shortcuts,
                text("").size(10),
                graph_shortcuts,
                text("").size(10),
                functions,
                text("").size(15),
                close_btn,
            ]
            .spacing(2)
            .padding(20)
        )
        .width(Fill)
        .height(Fill)
        .style(|_theme| container::Style {
            background: Some(colors::SURFACE.into()),
            ..Default::default()
        })
        .into()
    }

    fn tab_button(&self, label: &'static str, mode: Mode) -> Element<'_, Message> {
        let is_active = self.mode == mode;
        let bg_color = if is_active {
            colors::TAB_ACTIVE
        } else {
            colors::TAB_INACTIVE
        };
        let text_color = if is_active {
            colors::BACKGROUND
        } else {
            colors::TEXT
        };

        button(
            text(label)
                .size(16)
                .font(Font::MONOSPACE)
                .color(text_color)
                .center(),
        )
        .width(80)
        .height(35)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered if !is_active => lighten(bg_color, 0.1),
                button::Status::Pressed => colors::TEXT,
                _ => bg_color,
            };
            button::Style {
                background: Some(bg.into()),
                text_color,
                border: iced::Border {
                    color: colors::TEXT,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::SetMode(mode))
        .into()
    }

    fn view_calculator(&self) -> Element<'_, Message> {
        let input_display = container(
            text(if self.calc_input.is_empty() {
                "0"
            } else {
                &self.calc_input
            })
            .size(32)
            .font(Font::MONOSPACE)
            .color(colors::TEXT),
        )
        .padding(20)
        .width(Fill)
        .style(|_theme| container::Style {
            background: Some(colors::SURFACE.into()),
            border: iced::Border {
                color: colors::TEXT_DIM,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        let result_text = if let Some(ref err) = self.calc_error {
            format!("ERR: {}", err)
        } else if self.calc_result.is_empty() {
            String::new()
        } else {
            format!("= {}", self.calc_result)
        };
        let show_copy = !self.calc_result.is_empty() && self.calc_error.is_none();

        let result_display = container(
            row![
                text(result_text)
                    .size(28)
                    .font(Font::MONOSPACE)
                    .color(if self.calc_error.is_some() {
                        colors::ERROR
                    } else {
                        colors::SUCCESS
                    })
                    .width(Fill),
                if show_copy {
                    container(
                        button(text("Copy").size(12).font(Font::MONOSPACE).color(colors::TEXT))
                            .style(|_theme, _status| button::Style {
                                background: Some(colors::BUTTON.into()),
                                text_color: colors::TEXT,
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 1.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            })
                            .padding([4, 8])
                            .on_press(Message::CopyResult)
                    )
                } else {
                    container(text(""))
                }
            ]
            .spacing(8)
        )
        .padding(15)
        .width(Fill)
        .style(|_theme| container::Style {
            background: Some(colors::SURFACE.into()),
            border: iced::Border {
                color: colors::TEXT_DIM,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        // Memory indicator
        let memory_indicator = if self.memory != 0.0 {
            text(format!("M: {}", self.memory))
                .size(12)
                .font(Font::MONOSPACE)
                .color(colors::ACCENT_TEAL)
        } else {
            text("")
                .size(12)
                .font(Font::MONOSPACE)
                .color(colors::TEXT_DIM)
        };

        // Memory row
        let memory_row = self.button_row(vec![
            ("MC", ButtonType::Memory),
            ("MR", ButtonType::Memory),
            ("M+", ButtonType::Memory),
            ("M-", ButtonType::Memory),
            ("Ans", ButtonType::Memory),
        ]);

        let scientific_row = self.button_row(vec![
            ("sin", ButtonType::Scientific),
            ("cos", ButtonType::Scientific),
            ("tan", ButtonType::Scientific),
            ("log", ButtonType::Scientific),
            ("ln", ButtonType::Scientific),
        ]);

        let scientific_row2 = self.button_row(vec![
            ("asin", ButtonType::Scientific),
            ("acos", ButtonType::Scientific),
            ("atan", ButtonType::Scientific),
            ("abs", ButtonType::Scientific),
            ("!", ButtonType::Scientific),
        ]);

        let functions_row = self.button_row(vec![
            ("√", ButtonType::Scientific),
            ("^", ButtonType::Operator),
            ("(", ButtonType::Utility),
            (")", ButtonType::Utility),
            ("%", ButtonType::Operator),
        ]);

        let row_7_8_9 = self.button_row(vec![
            ("7", ButtonType::Number),
            ("8", ButtonType::Number),
            ("9", ButtonType::Number),
            ("÷", ButtonType::Operator),
            ("C", ButtonType::Clear),
        ]);

        let row_4_5_6 = self.button_row(vec![
            ("4", ButtonType::Number),
            ("5", ButtonType::Number),
            ("6", ButtonType::Number),
            ("×", ButtonType::Operator),
            ("⌫", ButtonType::Utility),
        ]);

        let row_1_2_3 = self.button_row(vec![
            ("1", ButtonType::Number),
            ("2", ButtonType::Number),
            ("3", ButtonType::Number),
            ("−", ButtonType::Operator),
            ("", ButtonType::Number), // placeholder
        ]);

        let row_0_dot = self.button_row(vec![
            ("0", ButtonType::Number),
            (".", ButtonType::Number),
            ("=", ButtonType::Equals),
            ("+", ButtonType::Operator),
            ("", ButtonType::Number), // placeholder
        ]);

        column![
            memory_indicator,
            input_display,
            result_display,
            memory_row,
            scientific_row,
            scientific_row2,
            functions_row,
            row_7_8_9,
            row_4_5_6,
            row_1_2_3,
            row_0_dot,
        ]
        .spacing(6)
        .padding(15)
        .into()
    }

    fn view_graph(&self) -> Element<'_, Message> {
        // Graph type selector with derivative toggle
        let deriv_btn_color = if self.show_derivative {
            colors::ACCENT_TEAL
        } else {
            colors::BUTTON
        };
        let deriv_text_color = if self.show_derivative {
            colors::BACKGROUND
        } else {
            colors::TEXT
        };

        let type_selector = row![
            self.graph_type_button("f(x)", GraphType::Function),
            self.graph_type_button("Parametric", GraphType::Parametric),
            button(
                text("f'(x)")
                    .size(14)
                    .font(Font::MONOSPACE)
                    .color(deriv_text_color)
                    .center(),
            )
            .width(60)
            .height(30)
            .style(move |_theme, _status| button::Style {
                background: Some(deriv_btn_color.into()),
                text_color: deriv_text_color,
                border: iced::Border {
                    color: colors::TEXT,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .on_press(Message::ToggleDerivative),
        ]
        .spacing(4);

        // Expression input(s)
        let expr_inputs: Element<'_, Message> = match self.graph_type {
            GraphType::Function => {
                let mut col = column![
                    row![
                        text("y₁= ").size(16).font(Font::MONOSPACE).color(graph::canvas::CURVE_COLORS[0]),
                        text_input("sin(x)", &self.graph_expr)
                            .on_input(Message::GraphExprChanged)
                            .size(16)
                            .font(Font::MONOSPACE)
                            .width(Fill)
                            .style(|_theme, _status| text_input::Style {
                                background: colors::SURFACE.into(),
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                icon: colors::TEXT,
                                placeholder: colors::TEXT_DIM,
                                value: colors::TEXT,
                                selection: colors::ACCENT_TEAL,
                            }),
                    ]
                    .spacing(4)
                ].spacing(4);

                // Additional curves
                for (i, expr) in self.additional_curves.iter().enumerate() {
                    let color = graph::canvas::CURVE_COLORS[(i + 1) % 5];
                    let idx = i;
                    col = col.push(
                        row![
                            text(format!("y{}= ", i + 2)).size(16).font(Font::MONOSPACE).color(color),
                            text_input("x^2", expr)
                                .on_input(move |s| Message::UpdateCurve(idx, s))
                                .size(16)
                                .font(Font::MONOSPACE)
                                .width(Fill)
                                .style(|_theme, _status| text_input::Style {
                                    background: colors::SURFACE.into(),
                                    border: iced::Border {
                                        color: colors::TEXT_DIM,
                                        width: 2.0,
                                        radius: 0.0.into(),
                                    },
                                    icon: colors::TEXT,
                                    placeholder: colors::TEXT_DIM,
                                    value: colors::TEXT,
                                    selection: colors::ACCENT_TEAL,
                                }),
                            button(text("×").size(14).color(colors::TEXT))
                                .style(|_theme, _status| button::Style {
                                    background: Some(colors::ACCENT_RED.into()),
                                    text_color: colors::TEXT,
                                    border: iced::Border::default(),
                                    ..Default::default()
                                })
                                .padding([2, 8])
                                .on_press(Message::RemoveCurve(idx)),
                        ]
                        .spacing(4)
                    );
                }

                // Add curve button (max 4 additional)
                if self.additional_curves.len() < 4 {
                    col = col.push(
                        button(text("+ Add curve").size(12).font(Font::MONOSPACE).color(colors::TEXT))
                            .style(|_theme, _status| button::Style {
                                background: Some(colors::SURFACE.into()),
                                text_color: colors::TEXT,
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 1.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            })
                            .padding([4, 8])
                            .on_press(Message::AddCurve)
                    );
                }

                col.into()
            }
            GraphType::Parametric => {
                column![
                    row![
                        text("x(t) = ").size(16).font(Font::MONOSPACE).color(colors::TEXT),
                        text_input("cos(t)", &self.graph_expr_x)
                            .on_input(Message::GraphExprXChanged)
                            .size(16)
                            .font(Font::MONOSPACE)
                            .width(Fill)
                            .style(|_theme, _status| text_input::Style {
                                background: colors::SURFACE.into(),
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                icon: colors::TEXT,
                                placeholder: colors::TEXT_DIM,
                                value: colors::TEXT,
                                selection: colors::ACCENT_TEAL,
                            }),
                    ]
                    .spacing(8),
                    row![
                        text("y(t) = ").size(16).font(Font::MONOSPACE).color(colors::TEXT),
                        text_input("sin(t)", &self.graph_expr_y)
                            .on_input(Message::GraphExprYChanged)
                            .size(16)
                            .font(Font::MONOSPACE)
                            .width(Fill)
                            .style(|_theme, _status| text_input::Style {
                                background: colors::SURFACE.into(),
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                icon: colors::TEXT,
                                placeholder: colors::TEXT_DIM,
                                value: colors::TEXT,
                                selection: colors::ACCENT_TEAL,
                            }),
                    ]
                    .spacing(8),
                    row![
                        text("t: ").size(14).font(Font::MONOSPACE).color(colors::TEXT_DIM),
                        text_input("0", &format!("{:.2}", self.t_range.0))
                            .on_input(Message::SetTRangeStart)
                            .size(14)
                            .font(Font::MONOSPACE)
                            .width(60)
                            .style(|_theme, _status| text_input::Style {
                                background: colors::SURFACE.into(),
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 1.0,
                                    radius: 0.0.into(),
                                },
                                icon: colors::TEXT,
                                placeholder: colors::TEXT_DIM,
                                value: colors::TEXT,
                                selection: colors::ACCENT_TEAL,
                            }),
                        text(" to ").size(14).font(Font::MONOSPACE).color(colors::TEXT_DIM),
                        text_input("2π", &format!("{:.2}", self.t_range.1))
                            .on_input(Message::SetTRangeEnd)
                            .size(14)
                            .font(Font::MONOSPACE)
                            .width(60)
                            .style(|_theme, _status| text_input::Style {
                                background: colors::SURFACE.into(),
                                border: iced::Border {
                                    color: colors::TEXT_DIM,
                                    width: 1.0,
                                    radius: 0.0.into(),
                                },
                                icon: colors::TEXT,
                                placeholder: colors::TEXT_DIM,
                                value: colors::TEXT,
                                selection: colors::ACCENT_TEAL,
                            }),
                    ]
                    .spacing(4),
                ]
                .spacing(4)
                .into()
            }
        };

        // Canvas with mouse event handling
        let canvas: Element<'_, Message> = iced::widget::mouse_area(
            Canvas::new(&self.graph_canvas)
                .width(Fill)
                .height(350),
        )
        .on_press(Message::CanvasMouseDown)
        .on_release(Message::CanvasMouseUp)
        .on_move(Message::CanvasMouseMove)
        .on_scroll(Message::CanvasScroll)
        .into();

        // Status bar
        let status_text = if let Some((x, y)) = self.cursor_world {
            format!("x: {:.4}  y: {:.4}", x, y)
        } else {
            "Drag to pan, scroll to zoom, R to reset".to_string()
        };

        let status_bar = container(
            row![
                text(status_text)
                    .size(14)
                    .font(Font::MONOSPACE)
                    .color(colors::TEXT_DIM)
                    .width(Fill),
                button(
                    text("Export")
                        .size(12)
                        .font(Font::MONOSPACE)
                        .color(colors::BACKGROUND)
                )
                .style(|_theme, _status| button::Style {
                    background: Some(colors::ACCENT_ORANGE.into()),
                    text_color: colors::BACKGROUND,
                    border: iced::Border {
                        color: colors::TEXT,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::ExportGraph)
                .padding([4, 8]),
                button(
                    text("Reset")
                        .size(12)
                        .font(Font::MONOSPACE)
                        .color(colors::BACKGROUND)
                )
                .style(|_theme, _status| button::Style {
                    background: Some(colors::ACCENT_TEAL.into()),
                    text_color: colors::BACKGROUND,
                    border: iced::Border {
                        color: colors::TEXT,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::GraphResetView)
                .padding([4, 8]),
            ]
            .spacing(10),
        )
        .padding(10)
        .width(Fill)
        .style(|_theme| container::Style {
            background: Some(colors::SURFACE.into()),
            border: iced::Border {
                color: colors::TEXT_DIM,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        column![type_selector, expr_inputs, canvas, status_bar,]
            .spacing(8)
            .padding(20)
            .into()
    }

    fn graph_type_button(&self, label: &'static str, graph_type: GraphType) -> Element<'_, Message> {
        let is_active = self.graph_type == graph_type;
        let bg_color = if is_active {
            colors::ACCENT_TEAL
        } else {
            colors::BUTTON
        };
        let text_color = if is_active {
            colors::BACKGROUND
        } else {
            colors::TEXT
        };

        button(
            text(label)
                .size(14)
                .font(Font::MONOSPACE)
                .color(text_color)
                .center(),
        )
        .width(120)
        .height(30)
        .style(move |_theme, _status| button::Style {
            background: Some(bg_color.into()),
            text_color,
            border: iced::Border {
                color: colors::TEXT,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .on_press(Message::SetGraphType(graph_type))
        .into()
    }

    fn button_row(&self, buttons: Vec<(&'static str, ButtonType)>) -> Row<'static, Message> {
        let mut row_widgets: Vec<Element<'static, Message>> = Vec::new();

        for (label, btn_type) in buttons {
            row_widgets.push(self.calc_button(label, btn_type));
        }

        Row::with_children(row_widgets).spacing(6)
    }

    fn calc_button(&self, label: &'static str, btn_type: ButtonType) -> Element<'static, Message> {
        // Handle empty placeholder buttons
        if label.is_empty() {
            return container(text(""))
                .width(Fill)
                .height(50)
                .into();
        }

        let message = match label {
            "C" => Message::CalcClear,
            "⌫" => Message::CalcBackspace,
            "=" => Message::CalcEvaluate,
            // Memory buttons
            "MC" => Message::MemoryClear,
            "MR" => Message::MemoryRecall,
            "M+" => Message::MemoryAdd,
            "M-" => Message::MemorySubtract,
            "Ans" => Message::InsertAns,
            // Scientific functions
            "sin" | "cos" | "tan" | "log" | "ln" | "asin" | "acos" | "atan" | "abs" => {
                Message::CalcFunction(label.to_string())
            }
            "√" => Message::CalcFunction("sqrt".to_string()),
            "!" => Message::CalcInput('!'),
            // Operators
            "÷" => Message::CalcInput('/'),
            "×" => Message::CalcInput('*'),
            "−" => Message::CalcInput('-'),
            "%" => Message::CalcInput('%'),
            _ => Message::CalcInput(label.chars().next().unwrap_or(' ')),
        };

        let base_color = match btn_type {
            ButtonType::Number => colors::BUTTON,
            ButtonType::Operator => colors::ACCENT_ORANGE,
            ButtonType::Scientific => colors::ACCENT_TEAL,
            ButtonType::Clear => colors::ACCENT_RED,
            ButtonType::Utility => colors::BUTTON,
            ButtonType::Memory => colors::SURFACE,
            ButtonType::Equals => colors::SUCCESS,
        };

        let text_color = match btn_type {
            ButtonType::Operator | ButtonType::Scientific | ButtonType::Clear | ButtonType::Equals => {
                colors::BACKGROUND
            }
            _ => colors::TEXT,
        };

        button(
            text(label)
                .size(20)
                .font(Font::MONOSPACE)
                .color(text_color)
                .center(),
        )
        .width(Fill)
        .height(50)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Active => base_color,
                button::Status::Hovered => lighten(base_color, 0.15),
                button::Status::Pressed => colors::TEXT,
                button::Status::Disabled => colors::BUTTON,
            };
            let fg = if matches!(status, button::Status::Pressed) {
                colors::BACKGROUND
            } else {
                text_color
            };
            button::Style {
                background: Some(bg.into()),
                text_color: fg,
                border: iced::Border {
                    color: colors::TEXT,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(message)
        .into()
    }
}

#[derive(Clone, Copy)]
enum ButtonType {
    Number,
    Operator,
    Scientific,
    Clear,
    Utility,
    Memory,
    Equals,
}

fn lighten(color: iced::Color, amount: f32) -> iced::Color {
    iced::Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

/// Draw a line on an image using Bresenham's algorithm
fn draw_line(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut x = x0;
    let mut y = y0;

    let width = img.width() as i32;
    let height = img.height() as i32;

    loop {
        if x >= 0 && x < width && y >= 0 && y < height {
            img.put_pixel(x as u32, y as u32, color);
        }

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
