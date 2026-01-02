# RustyAbacus

A native cross-platform scientific calculator and graphing tool with brutalist aesthetics, built in Rust using iced.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

### Calculator
- **Scientific functions**: sin, cos, tan, asin, acos, atan, log, ln, sqrt, abs, floor, ceil
- **Operators**: +, -, *, /, ^, %, !
- **Constants**: pi, e
- **Memory**: M+, M-, MR, MC
- **History**: Ans (use previous result)
- **Clipboard**: Copy result with one click
- Real-time evaluation as you type

### Graphing
- **Function plotting**: Plot y = f(x) with up to 5 curves in different colors
- **Parametric curves**: Plot (x(t), y(t)) with adjustable t-range
- **Derivative visualization**: Toggle f'(x) to see the derivative
- **Interactive controls**: Pan, zoom, trace mode with coordinate display
- **Export**: Save graphs as PNG images

### Design
- Brutalist aesthetic with dark theme
- High contrast colors
- Sharp geometry, no rounded corners
- Monospace typography

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/williamshayden/RustyAbacus.git
cd RustyAbacus

# Build and run
cargo run --release
```

### Requirements
- Rust 1.70+
- macOS, Windows, or Linux

## Usage

### Calculator Mode

| Key | Action |
|-----|--------|
| 0-9 | Enter digits |
| +, -, *, / | Operators |
| ^ | Power |
| % | Modulo |
| ! | Factorial |
| ( ) | Parentheses |
| Backspace | Delete last |
| Escape | Clear |
| Tab | Switch to Graph |

### Graph Mode

| Control | Action |
|---------|--------|
| Mouse drag | Pan view |
| Scroll wheel | Zoom in/out |
| +/= | Zoom in |
| - | Zoom out |
| Arrow keys | Pan |
| R | Reset view |
| Tab | Switch to Calculator |

Press **?** for keyboard shortcuts help.

## Example Expressions

### Calculator
```
2 + 3 * 4       → 14
sin(pi/2)       → 1
sqrt(16) + 2^3  → 12
5!              → 120
abs(0-5)        → 5
```

### Graph (Function mode)
```
sin(x)              - Sine wave
x^2                 - Parabola
sin(x) * cos(2*x)   - Complex wave
1/x                 - Hyperbola
```

### Graph (Parametric mode)
```
x(t) = cos(t), y(t) = sin(t)       - Circle
x(t) = cos(t), y(t) = sin(2*t)     - Figure-8
x(t) = t*cos(t), y(t) = t*sin(t)   - Spiral
```

## Architecture

```
src/
├── main.rs           # Application, UI, state management
├── calculator.rs     # Expression parser (shunting-yard algorithm)
└── graph/
    ├── mod.rs        # Module exports
    ├── canvas.rs     # Graph rendering with iced Canvas
    ├── evaluator.rs  # Function sampling, derivatives
    └── viewport.rs   # Pan/zoom coordinate transforms
```

## Testing

```bash
cargo test
```

Tests cover arithmetic, scientific functions, factorial, modulo, and variable substitution.

## Dependencies

- [iced](https://github.com/iced-rs/iced) - Cross-platform GUI framework
- [arboard](https://github.com/1Password/arboard) - Clipboard support
- [image](https://github.com/image-rs/image) - PNG export
- [rfd](https://github.com/PolyMeilex/rfd) - Native file dialogs

## License

MIT License - see LICENSE for details.
