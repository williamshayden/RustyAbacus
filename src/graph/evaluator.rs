/// Evaluates expressions over a range and returns plot points

use crate::calculator;

/// A point that may or may not be valid (for handling discontinuities)
#[derive(Clone, Copy)]
pub struct PlotPoint {
    pub x: f64,
    pub y: f64,
    pub valid: bool,
}

/// Sample a function y = f(x) over a range
pub fn sample_function(expr: &str, x_min: f64, x_max: f64, num_samples: usize) -> Vec<PlotPoint> {
    let mut points = Vec::with_capacity(num_samples);
    let step = (x_max - x_min) / (num_samples - 1) as f64;

    for i in 0..num_samples {
        let x = x_min + i as f64 * step;
        let point = match calculator::evaluate_with_var(expr, 'x', x) {
            Ok(y) if y.is_finite() => PlotPoint { x, y, valid: true },
            _ => PlotPoint {
                x,
                y: 0.0,
                valid: false,
            },
        };
        points.push(point);
    }

    points
}

/// Sample a parametric curve (x(t), y(t)) over a range
pub fn sample_parametric(
    expr_x: &str,
    expr_y: &str,
    t_min: f64,
    t_max: f64,
    num_samples: usize,
) -> Vec<PlotPoint> {
    let mut points = Vec::with_capacity(num_samples);
    let step = (t_max - t_min) / (num_samples - 1) as f64;

    for i in 0..num_samples {
        let t = t_min + i as f64 * step;
        let x_result = calculator::evaluate_with_var(expr_x, 't', t);
        let y_result = calculator::evaluate_with_var(expr_y, 't', t);

        let point = match (x_result, y_result) {
            (Ok(x), Ok(y)) if x.is_finite() && y.is_finite() => PlotPoint { x, y, valid: true },
            _ => PlotPoint {
                x: 0.0,
                y: 0.0,
                valid: false,
            },
        };
        points.push(point);
    }

    points
}

/// Sample the numerical derivative of a function
pub fn sample_derivative(expr: &str, x_min: f64, x_max: f64, num_samples: usize) -> Vec<PlotPoint> {
    let mut points = Vec::with_capacity(num_samples);
    let step = (x_max - x_min) / (num_samples - 1) as f64;
    let h = step * 0.01; // Small step for numerical derivative

    for i in 0..num_samples {
        let x = x_min + i as f64 * step;

        // Central difference: f'(x) ≈ (f(x+h) - f(x-h)) / (2h)
        let f_plus = calculator::evaluate_with_var(expr, 'x', x + h);
        let f_minus = calculator::evaluate_with_var(expr, 'x', x - h);

        let point = match (f_plus, f_minus) {
            (Ok(y_plus), Ok(y_minus)) if y_plus.is_finite() && y_minus.is_finite() => {
                let derivative = (y_plus - y_minus) / (2.0 * h);
                if derivative.is_finite() {
                    PlotPoint { x, y: derivative, valid: true }
                } else {
                    PlotPoint { x, y: 0.0, valid: false }
                }
            }
            _ => PlotPoint { x, y: 0.0, valid: false },
        };
        points.push(point);
    }

    points
}

/// Find a good y-range for the given points (for auto-scaling)
pub fn find_y_range(points: &[PlotPoint]) -> Option<(f64, f64)> {
    let valid_ys: Vec<f64> = points.iter().filter(|p| p.valid).map(|p| p.y).collect();

    if valid_ys.is_empty() {
        return None;
    }

    let min_y = valid_ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = valid_ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Add some padding
    let range = max_y - min_y;
    let padding = if range < 0.001 { 1.0 } else { range * 0.1 };

    Some((min_y - padding, max_y + padding))
}
