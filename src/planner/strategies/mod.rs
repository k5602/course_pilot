/*!
Strategies module for the scheduler.

This module groups per-strategy planning implementations into focused files
and re-exports their public entry points for convenient consumption.

New strategies should live in their own file within this directory and be
re-exported from here.
*/

mod module_based;

// Re-exports (visibility follows the inner item visibility)
pub use module_based::generate_module_based_plan;

mod time_based;
pub use time_based::generate_time_based_plan;

mod hybrid;
pub use hybrid::generate_hybrid_plan;

mod difficulty_based;
pub use difficulty_based::generate_difficulty_based_plan;

mod spaced_repetition;
pub use spaced_repetition::generate_spaced_repetition_plan;

pub mod adaptive;
pub use adaptive::generate_adaptive_plan;
