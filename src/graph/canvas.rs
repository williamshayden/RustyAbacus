/// Canvas rendering for the graph

use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{mouse, Color, Point, Rectangle, Renderer, Theme};

use super::evaluator::PlotPoint;
use super::viewport::Viewport;

// Graph colors
pub mod colors {
    use iced::Color;

    pub const BACKGROUND: Color = Color::from_rgb(0.1, 0.1, 0.1);
    pub const GRID: Color = Color::from_rgb(0.2, 0.2, 0.2);
    pub const AXIS: Color = Color::WHITE;
    pub const CURVE: Color = Color::from_rgb(0.31, 0.8, 0.77); // Teal
    pub const PARAMETRIC: Color = Color::from_rgb(1.0, 0.42, 0.21); // Orange
    pub const CROSSHAIR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.5);
    pub const LABEL: Color = Color::from_rgb(0.6, 0.6, 0.6);
}

pub struct GraphCanvas {
    cache: Cache,
    pub viewport: Viewport,
    pub points: Vec<PlotPoint>,
    pub parametric_points: Vec<PlotPoint>,
    pub additional_curves: Vec<Vec<PlotPoint>>, // Multiple additional curves
    pub derivative_points: Vec<PlotPoint>,       // Derivative of main function
    pub cursor_position: Option<Point>,
    pub is_parametric: bool,
    pub show_derivative: bool,
}

// Additional curve colors
pub const CURVE_COLORS: [Color; 5] = [
    Color::from_rgb(0.31, 0.8, 0.77),  // Teal (primary)
    Color::from_rgb(0.95, 0.6, 0.2),   // Orange
    Color::from_rgb(0.7, 0.4, 0.9),    // Purple
    Color::from_rgb(0.2, 0.8, 0.4),    // Green
    Color::from_rgb(0.9, 0.3, 0.5),    // Pink
];

impl Default for GraphCanvas {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            viewport: Viewport::default(),
            points: Vec::new(),
            parametric_points: Vec::new(),
            additional_curves: Vec::new(),
            derivative_points: Vec::new(),
            cursor_position: None,
            is_parametric: false,
            show_derivative: false,
        }
    }
}

impl GraphCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_points(&mut self, points: Vec<PlotPoint>) {
        self.points = points;
        self.cache.clear();
    }

    pub fn set_parametric_points(&mut self, points: Vec<PlotPoint>) {
        self.parametric_points = points;
        self.cache.clear();
    }

    pub fn set_cursor(&mut self, position: Option<Point>) {
        self.cursor_position = position;
        self.cache.clear();
    }

    pub fn set_additional_curves(&mut self, curves: Vec<Vec<PlotPoint>>) {
        self.additional_curves = curves;
        self.cache.clear();
    }

    pub fn set_derivative(&mut self, points: Vec<PlotPoint>, show: bool) {
        self.derivative_points = points;
        self.show_derivative = show;
        self.cache.clear();
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl<Message> canvas::Program<Message> for GraphCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Background
            frame.fill_rectangle(
                Point::ORIGIN,
                frame.size(),
                colors::BACKGROUND,
            );

            // Draw grid
            self.draw_grid(frame);

            // Draw axes
            self.draw_axes(frame);

            // Draw curve(s)
            if self.is_parametric {
                self.draw_curve(frame, &self.parametric_points, colors::PARAMETRIC);
            } else {
                // Draw main curve
                self.draw_curve(frame, &self.points, CURVE_COLORS[0]);
                // Draw additional curves
                for (i, curve_points) in self.additional_curves.iter().enumerate() {
                    let color = CURVE_COLORS[(i + 1) % CURVE_COLORS.len()];
                    self.draw_curve(frame, curve_points, color);
                }
                // Draw derivative (dashed, lighter color)
                if self.show_derivative && !self.derivative_points.is_empty() {
                    let deriv_color = Color::from_rgba(0.9, 0.5, 0.9, 0.8); // Light purple
                    self.draw_dashed_curve(frame, &self.derivative_points, deriv_color);
                }
            }

            // Draw crosshair at cursor
            if let Some(cursor_pos) = self.cursor_position {
                self.draw_crosshair(frame, cursor_pos);
            }
        });

        vec![geometry]
    }
}

impl GraphCanvas {
    fn draw_grid(&self, frame: &mut Frame) {
        let (x_min, x_max) = self.viewport.x_range();
        let (y_min, y_max) = self.viewport.y_range();

        // Calculate grid spacing based on scale
        let grid_spacing = calculate_grid_spacing(self.viewport.scale);

        let stroke = Stroke::default()
            .with_color(colors::GRID)
            .with_width(1.0);

        // Vertical lines
        let x_start = (x_min / grid_spacing).floor() * grid_spacing;
        let mut x = x_start;
        while x <= x_max {
            let (screen_x, _) = self.viewport.world_to_screen(x, 0.0);
            let path = Path::line(
                Point::new(screen_x, 0.0),
                Point::new(screen_x, self.viewport.height),
            );
            frame.stroke(&path, stroke.clone());
            x += grid_spacing;
        }

        // Horizontal lines
        let y_start = (y_min / grid_spacing).floor() * grid_spacing;
        let mut y = y_start;
        while y <= y_max {
            let (_, screen_y) = self.viewport.world_to_screen(0.0, y);
            let path = Path::line(
                Point::new(0.0, screen_y),
                Point::new(self.viewport.width, screen_y),
            );
            frame.stroke(&path, stroke.clone());
            y += grid_spacing;
        }
    }

    fn draw_axes(&self, frame: &mut Frame) {
        let stroke = Stroke::default()
            .with_color(colors::AXIS)
            .with_width(2.0);

        // X axis
        let (_, y_origin) = self.viewport.world_to_screen(0.0, 0.0);
        if y_origin >= 0.0 && y_origin <= self.viewport.height {
            let path = Path::line(
                Point::new(0.0, y_origin),
                Point::new(self.viewport.width, y_origin),
            );
            frame.stroke(&path, stroke.clone());
        }

        // Y axis
        let (x_origin, _) = self.viewport.world_to_screen(0.0, 0.0);
        if x_origin >= 0.0 && x_origin <= self.viewport.width {
            let path = Path::line(
                Point::new(x_origin, 0.0),
                Point::new(x_origin, self.viewport.height),
            );
            frame.stroke(&path, stroke);
        }

        // Axis labels
        self.draw_axis_labels(frame);
    }

    fn draw_axis_labels(&self, frame: &mut Frame) {
        let (x_min, x_max) = self.viewport.x_range();
        let (y_min, y_max) = self.viewport.y_range();
        let grid_spacing = calculate_grid_spacing(self.viewport.scale);

        // X axis labels
        let x_start = (x_min / grid_spacing).ceil() * grid_spacing;
        let mut x = x_start;
        while x <= x_max {
            if x.abs() > 0.001 {
                // Skip zero
                let (screen_x, screen_y) = self.viewport.world_to_screen(x, 0.0);
                let label_y = screen_y.clamp(10.0, self.viewport.height - 20.0);
                let text = Text {
                    content: format_number(x),
                    position: Point::new(screen_x - 10.0, label_y + 5.0),
                    color: colors::LABEL,
                    size: iced::Pixels(12.0),
                    ..Default::default()
                };
                frame.fill_text(text);
            }
            x += grid_spacing;
        }

        // Y axis labels
        let y_start = (y_min / grid_spacing).ceil() * grid_spacing;
        let mut y = y_start;
        while y <= y_max {
            if y.abs() > 0.001 {
                // Skip zero
                let (screen_x, screen_y) = self.viewport.world_to_screen(0.0, y);
                let label_x = screen_x.clamp(5.0, self.viewport.width - 40.0);
                let text = Text {
                    content: format_number(y),
                    position: Point::new(label_x + 5.0, screen_y - 6.0),
                    color: colors::LABEL,
                    size: iced::Pixels(12.0),
                    ..Default::default()
                };
                frame.fill_text(text);
            }
            y += grid_spacing;
        }
    }

    fn draw_curve(&self, frame: &mut Frame, points: &[PlotPoint], color: Color) {
        if points.is_empty() {
            return;
        }

        let stroke = Stroke::default().with_color(color).with_width(2.0);

        // Build path segments (break at invalid points)
        let mut path_builder = canvas::path::Builder::new();
        let mut in_path = false;

        for point in points {
            if point.valid {
                let (sx, sy) = self.viewport.world_to_screen(point.x, point.y);

                // Skip points way off screen
                if sy > -1000.0 && sy < self.viewport.height + 1000.0 {
                    if in_path {
                        path_builder.line_to(Point::new(sx, sy));
                    } else {
                        path_builder.move_to(Point::new(sx, sy));
                        in_path = true;
                    }
                } else {
                    in_path = false;
                }
            } else {
                in_path = false;
            }
        }

        let path = path_builder.build();
        frame.stroke(&path, stroke);
    }

    fn draw_dashed_curve(&self, frame: &mut Frame, points: &[PlotPoint], color: Color) {
        if points.is_empty() {
            return;
        }

        let stroke = Stroke::default().with_color(color).with_width(1.5);
        let dash_len = 8.0;
        let gap_len = 4.0;

        let mut accumulated = 0.0;
        let mut drawing = true;
        let mut last_point: Option<Point> = None;

        for point in points {
            if point.valid {
                let (sx, sy) = self.viewport.world_to_screen(point.x, point.y);
                let current = Point::new(sx, sy);

                if sy > -1000.0 && sy < self.viewport.height + 1000.0 {
                    if let Some(prev) = last_point {
                        let dx = current.x - prev.x;
                        let dy = current.y - prev.y;
                        let segment_len = (dx * dx + dy * dy).sqrt();

                        if drawing {
                            let path = Path::line(prev, current);
                            frame.stroke(&path, stroke.clone());
                        }

                        accumulated += segment_len;
                        if drawing && accumulated >= dash_len {
                            drawing = false;
                            accumulated = 0.0;
                        } else if !drawing && accumulated >= gap_len {
                            drawing = true;
                            accumulated = 0.0;
                        }
                    }
                    last_point = Some(current);
                } else {
                    last_point = None;
                    drawing = true;
                    accumulated = 0.0;
                }
            } else {
                last_point = None;
                drawing = true;
                accumulated = 0.0;
            }
        }
    }

    fn draw_crosshair(&self, frame: &mut Frame, cursor_pos: Point) {
        let stroke = Stroke::default()
            .with_color(colors::CROSSHAIR)
            .with_width(1.0);

        // Vertical line
        let v_path = Path::line(
            Point::new(cursor_pos.x, 0.0),
            Point::new(cursor_pos.x, self.viewport.height),
        );
        frame.stroke(&v_path, stroke.clone());

        // Horizontal line
        let h_path = Path::line(
            Point::new(0.0, cursor_pos.y),
            Point::new(self.viewport.width, cursor_pos.y),
        );
        frame.stroke(&h_path, stroke);

        // If in function mode, find and highlight the point on the curve
        if !self.is_parametric && !self.points.is_empty() {
            let (world_x, _) = self.viewport.screen_to_world(cursor_pos.x, cursor_pos.y);

            // Find the closest point to our x position
            if let Some(closest) = self.find_closest_point(world_x) {
                let (sx, sy) = self.viewport.world_to_screen(closest.x, closest.y);

                // Draw a dot on the curve
                let dot = Path::circle(Point::new(sx, sy), 5.0);
                frame.fill(&dot, CURVE_COLORS[0]);

                // Draw the coordinates near the point
                let label = format!("({:.3}, {:.3})", closest.x, closest.y);
                let text = Text {
                    content: label,
                    position: Point::new(sx + 10.0, sy - 20.0),
                    color: Color::WHITE,
                    size: iced::Pixels(12.0),
                    ..Default::default()
                };
                frame.fill_text(text);
            }
        }
    }

    fn find_closest_point(&self, target_x: f64) -> Option<PlotPoint> {
        self.points
            .iter()
            .filter(|p| p.valid)
            .min_by(|a, b| {
                let diff_a = (a.x - target_x).abs();
                let diff_b = (b.x - target_x).abs();
                diff_a.partial_cmp(&diff_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
}

/// Calculate appropriate grid spacing based on zoom level
fn calculate_grid_spacing(scale: f64) -> f64 {
    let target_pixels = 80.0; // Desired pixels between grid lines
    let raw_spacing = target_pixels / scale;

    // Round to nice numbers (1, 2, 5, 10, 20, 50, etc.)
    let magnitude = 10f64.powf(raw_spacing.log10().floor());
    let normalized = raw_spacing / magnitude;

    let nice = if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    };

    nice * magnitude
}

/// Format a number for axis labels
fn format_number(n: f64) -> String {
    if n.abs() < 0.001 {
        "0".to_string()
    } else if n.abs() >= 1000.0 || n.abs() < 0.01 {
        format!("{:.1e}", n)
    } else if n.fract().abs() < 0.001 {
        format!("{:.0}", n)
    } else {
        format!("{:.2}", n)
    }
}
