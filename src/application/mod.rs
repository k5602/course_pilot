//! Application Layer - Use cases and orchestration.

use std::sync::Arc;

use crate::domain::ports::ModuleTitleGenerator;

pub mod context;
pub mod use_cases;

pub use context::{AppConfig, AppConfigBuilder, AppContext, AppContextError, ServiceFactory};

/// Generate a module title using the LLM-backed generator, falling back to
/// the first video title or a generic "Module N" label.
pub(crate) async fn generate_module_title(
    generator: Option<&Arc<dyn ModuleTitleGenerator>>,
    titles: &[String],
    course_name: &str,
    module_idx: usize,
) -> String {
    if let Some(g) = generator
        && let Ok(title) = g.generate_module_title(titles, course_name, module_idx).await
        && !title.is_empty()
    {
        return title;
    }
    titles.first().cloned().unwrap_or_else(|| format!("Module {}", module_idx + 1))
}
