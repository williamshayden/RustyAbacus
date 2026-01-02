/// Graph module for function plotting

pub mod canvas;
pub mod evaluator;
pub mod viewport;

pub use canvas::GraphCanvas;
pub use evaluator::{sample_derivative, sample_function, sample_parametric};
