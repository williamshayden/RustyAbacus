# RustyAbacus

A native cross-platform scientific calculator and graphing tool with brutalist aesthetics, built in Rust using iced.

## Project Overview

A desktop application featuring:
- Native GUI using iced framework
- Brutalist design with bold typography and high contrast
- Scientific calculator with sin, cos, tan, log, ln, sqrt
- **2D function graphing**: Plot y = f(x)
- **Parametric curve plotting**: Plot (x(t), y(t))
- Interactive graph with pan, zoom, and coordinate tracing
- Cross-platform support (macOS, Windows, Linux)

## Modes

### Calculator Mode
Full scientific calculator with:
- Basic arithmetic: `+`, `-`, `*`, `/`
- Power: `^`
- Scientific functions: sin, cos, tan, log, ln, sqrt
- Constants: pi, e
- Real-time evaluation as you type

### Graph Mode
Interactive function plotting:
- **Function mode**: Plot y = f(x) expressions using variable `x`
- **Parametric mode**: Plot curves using x(t) and y(t) with variable `t`
- Pan by dragging
- Zoom with scroll wheel
- Coordinate display on hover
- Grid and axis labels

## Architecture

```
src/
├── main.rs           # iced Application, UI, tab navigation
├── calculator.rs     # Expression parser with variable support
└── graph/
    ├── mod.rs        # Module exports
    ├── canvas.rs     # Graph rendering with iced Canvas
    ├── evaluator.rs  # Expression sampling for plotting
    └── viewport.rs   # Pan/zoom coordinate transforms
```

## Brutalist Design

- **High contrast**: Dark background (#1a1a1a) with white text
- **Thick borders**: 2px white borders on all elements
- **No rounded corners**: Sharp, honest geometry
- **Bold colors**:
  - Teal (#4ecdc4): Scientific functions, active tabs, curves
  - Orange (#ff6b35): Operators, parametric curves
  - Red (#e54040): Clear button
  - Dark gray (#383838): Numbers and utilities
- **Monospace fonts**: Raw, technical aesthetic

## Controls

### Calculator Mode
| Key | Action |
|-----|--------|
| 0-9 | Enter digits |
| +, -, *, / | Operators |
| ^ | Power |
| ( ) | Parentheses |
| . | Decimal point |
| Backspace | Delete last character |
| Escape | Clear |
| Tab | Switch to Graph mode |
| Letters | Type function names (sin, cos, etc.) |

### Graph Mode
| Control | Action |
|---------|--------|
| Mouse drag | Pan the view |
| Scroll wheel | Zoom in/out |
| +/= | Zoom in |
| - | Zoom out |
| Arrow keys | Pan |
| R | Reset view |
| Tab | Switch to Calculator mode |

## Building and Running

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run the calculator
cargo run --release
```

## Dependencies

- `iced 0.13`: Cross-platform GUI framework with canvas support

## Example Expressions

### Calculator
- `2 + 3 * 4` → 14
- `sin(pi/2)` → 1
- `sqrt(16) + 2^3` → 12
- `log(100)` → 2
- `e^2` → 7.389...

### Graph (Function mode)
- `sin(x)` - Sine wave
- `x^2` - Parabola
- `sin(x) * cos(2*x)` - Complex wave
- `1/x` - Hyperbola (handles discontinuity)

### Graph (Parametric mode)
- `x(t) = cos(t), y(t) = sin(t)` - Circle
- `x(t) = cos(t), y(t) = sin(2*t)` - Figure-8
- `x(t) = t*cos(t), y(t) = t*sin(t)` - Spiral

## Code Structure

### main.rs
- `App` struct: Application state for both modes
- `Mode` enum: Calculator or Graph
- `GraphType` enum: Function or Parametric
- Tab navigation with brutalist styling
- Keyboard and mouse event handling

### calculator.rs
- `Token` enum: Numbers, operators, functions, variables
- `evaluate()`: Evaluate expressions without variables
- `evaluate_with_var()`: Evaluate with variable substitution
- Shunting-yard algorithm for operator precedence
- RPN evaluation with error handling

### graph/viewport.rs
- `Viewport` struct: Center, scale, dimensions
- `screen_to_world()`: Convert screen to math coordinates
- `world_to_screen()`: Convert math to screen coordinates
- `pan()`, `zoom()`, `reset()`: View manipulation

### graph/evaluator.rs
- `sample_function()`: Sample y = f(x) over range
- `sample_parametric()`: Sample (x(t), y(t)) over range
- Handles invalid points (division by zero, etc.)

### graph/canvas.rs
- `GraphCanvas`: Canvas state and cache
- Grid and axis rendering
- Curve drawing with discontinuity handling
- Crosshair at cursor position
- Axis labels with smart number formatting

## Testing

```bash
# Run unit tests
cargo test
```

Tests cover:
- Basic arithmetic operations
- Power operations
- Scientific functions
- Variable substitution
- Mathematical constants (pi, e)
