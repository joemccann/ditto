// Library entry-point — exposes internal modules for integration tests and
// potential future embedding.  The public surface is intentionally minimal;
// everything beyond `renderer` is considered an implementation detail.

pub mod cli;
pub mod doctor;
pub mod highlighter;
pub mod html;
pub mod renderer;
