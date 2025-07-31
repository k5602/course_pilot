//! Reactive state management for Course Pilot using dioxus-signals.

use crate::types::{
    AppState, ContextualPanelState, ContextualPanelTab, Course, CourseStructure, ImportJob,
    ImportStatus, Note, Plan, VideoContext,
};
use dioxus::prelude::*;
use uuid::Uuid;

/// Result type for state operations
pub type StateResult<T> = Result<T, StateError>;

/// Errors that can occur during state operations
#[derive(Debug, Clone)]
pub enum StateError {
    CourseNotFound(Uuid),
    InvalidOperation(String),
    NavigationError(String),
    ValidationError(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::CourseNotFound(id) => write!(f, "Course not found: {id}"),
            StateError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            StateError::NavigationError(msg) => write!(f, "Navigation error: {msg}"),
            StateError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for StateError {}

/// Course management context
#[derive(Clone, Copy)]
pub struct CourseContext {
    pub courses: Signal<Vec<Course>>,
}

impl Default for CourseContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CourseContext {
    pub fn new() -> Self {
        Self {
            courses: Signal::new(Vec::new()),
        }
    }
}

/// Notes management context
#[derive(Clone, Copy)]
pub struct NotesContext {
    pub notes: Signal<Vec<Note>>,
}

impl Default for NotesContext {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesContext {
    pub fn new() -> Self {
        Self {
            notes: Signal::new(Vec::new()),
        }
    }
}

/// Plans management context
#[derive(Clone, Copy)]
pub struct PlanContext {
    pub plans: Signal<Vec<Plan>>,
}

impl Default for PlanContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanContext {
    pub fn new() -> Self {
        Self {
            plans: Signal::new(Vec::new()),
        }
    }
}

/// Import management context
#[derive(Clone, Copy)]
pub struct ImportContext {
    pub active_import: Signal<Option<ImportJob>>,
}

impl Default for ImportContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportContext {
    pub fn new() -> Self {
        Self {
            active_import: Signal::new(None),
        }
    }
}

/// Contextual panel context
#[derive(Clone, Copy)]
pub struct ContextualPanelContext {
    pub state: Signal<ContextualPanelState>,
}

impl Default for ContextualPanelContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextualPanelContext {
    pub fn new() -> Self {
        Self {
            state: Signal::new(ContextualPanelState::default()),
        }
    }
}

/// Mobile sidebar context
#[derive(Clone, Copy)]
pub struct MobileSidebarContext {
    pub is_open: Signal<bool>,
}

impl Default for MobileSidebarContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MobileSidebarContext {
    pub fn new() -> Self {
        Self {
            is_open: Signal::new(false),
        }
    }
}

#[component]
pub fn CourseContextProvider(children: Element) -> Element {
    use_context_provider(CourseContext::new);
    rsx! { {children} }
}

#[component]
pub fn NotesContextProvider(children: Element) -> Element {
    use_context_provider(NotesContext::new);
    rsx! { {children} }
}

#[component]
pub fn PlanContextProvider(children: Element) -> Element {
    use_context_provider(PlanContext::new);
    rsx! { {children} }
}

#[component]
pub fn ImportContextProvider(children: Element) -> Element {
    use_context_provider(ImportContext::new);
    rsx! { {children} }
}

#[component]
pub fn ContextualPanelContextProvider(children: Element) -> Element {
    use_context_provider(ContextualPanelContext::new);
    rsx! { {children} }
}

#[component]
pub fn MobileSidebarContextProvider(children: Element) -> Element {
    use_context_provider(MobileSidebarContext::new);
    rsx! { {children} }
}

/// Hook for reactive access to courses
pub fn use_courses_reactive() -> Memo<Vec<Course>> {
    let course_context = use_context::<CourseContext>();
    use_memo(move || course_context.courses.read().clone())
}

/// Hook for reactive access to a specific course
pub fn use_course_reactive(id: Uuid) -> Memo<Option<Course>> {
    let course_context = use_context::<CourseContext>();
    use_memo(move || {
        course_context
            .courses
            .read()
            .iter()
            .find(|c| c.id == id)
            .cloned()
    })
}

/// Hook for reactive access to active import
pub fn use_active_import_reactive() -> Memo<Option<ImportJob>> {
    let import_context = use_context::<ImportContext>();
    use_memo(move || import_context.active_import.read().clone())
}

/// Hook for course statistics
pub fn use_course_stats_reactive() -> Memo<(usize, usize, usize)> {
    let course_context = use_context::<CourseContext>();
    use_memo(move || {
        let courses = &course_context.courses.read();
        let total = courses.len();
        let structured = courses.iter().filter(|c| c.is_structured()).count();
        let videos = courses.iter().map(|c| c.video_count()).sum();
        (total, structured, videos)
    })
}

/// Hook for reactive access to contextual panel
pub fn use_contextual_panel_reactive() -> Memo<ContextualPanelState> {
    let panel_context = use_context::<ContextualPanelContext>();
    use_memo(move || panel_context.state.read().clone())
}

/// Hook for reactive access to mobile sidebar
pub fn use_mobile_sidebar_reactive() -> Memo<bool> {
    let sidebar_context = use_context::<MobileSidebarContext>();
    use_memo(move || *sidebar_context.is_open.read())
}

/// Hook for reactive access to notes
pub fn use_notes_reactive() -> Memo<Vec<Note>> {
    let notes_context = use_context::<NotesContext>();
    use_memo(move || notes_context.notes.read().clone())
}

/// Hook for reactive access to plans
pub fn use_plans_reactive() -> Memo<Vec<Plan>> {
    let plan_context = use_context::<PlanContext>();
    use_memo(move || plan_context.plans.read().clone())
}

/// Hook for tag statistics from notes
pub fn use_tag_statistics_reactive() -> Memo<std::collections::HashMap<String, usize>> {
    let notes_context = use_context::<NotesContext>();
    use_memo(move || {
        let mut stats = std::collections::HashMap::new();
        for note in &*notes_context.notes.read() {
            for tag in &note.tags {
                *stats.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        stats
    })
}

// ============================================================================
// STATE OPERATIONS
// ============================================================================

/// Add a course to the state
pub fn add_course_reactive(course: Course) -> StateResult<()> {
    let mut course_context = use_context::<CourseContext>();

    // Validation
    if course.name.trim().is_empty() {
        return Err(StateError::ValidationError(
            "Course name cannot be empty".to_string(),
        ));
    }

    // Check for duplicate names
    {
        let courses = course_context.courses.read();
        if courses.iter().any(|c| c.name == course.name) {
            return Err(StateError::ValidationError(
                "Course with this name already exists".to_string(),
            ));
        }
    }

    // Atomic update
    course_context.courses.write().push(course);
    log::info!("Course added successfully");
    Ok(())
}

/// Update a course in the state
pub fn update_course_reactive(id: Uuid, updated_course: Course) -> StateResult<()> {
    let mut course_context = use_context::<CourseContext>();
    let mut courses = course_context.courses.write();

    let course_index = courses
        .iter()
        .position(|c| c.id == id)
        .ok_or(StateError::CourseNotFound(id))?;

    // Validation
    if updated_course.name.trim().is_empty() {
        return Err(StateError::ValidationError(
            "Course name cannot be empty".to_string(),
        ));
    }

    // Check for duplicate names (excluding current course)
    if courses
        .iter()
        .any(|c| c.id != id && c.name == updated_course.name)
    {
        return Err(StateError::ValidationError(
            "Course with this name already exists".to_string(),
        ));
    }

    // Atomic update
    courses[course_index] = updated_course;
    log::info!("Course {id} updated successfully");
    Ok(())
}

/// Delete a course from the state
pub fn delete_course_reactive(id: Uuid) -> StateResult<()> {
    let mut course_context = use_context::<CourseContext>();
    let mut courses = course_context.courses.write();
    let initial_len = courses.len();

    // Atomic update
    courses.retain(|c| c.id != id);

    if courses.len() == initial_len {
        return Err(StateError::CourseNotFound(id));
    }

    log::info!("Course {id} deleted successfully");
    Ok(())
}

/// Duplicate a course
pub fn duplicate_course_reactive(id: Uuid) -> StateResult<()> {
    let mut course_context = use_context::<CourseContext>();

    let original_course = {
        let courses = course_context.courses.read();
        courses
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or(StateError::CourseNotFound(id))?
    };

    // Create duplicate with new ID and unique name
    let mut duplicate = original_course;
    duplicate.id = Uuid::new_v4();
    duplicate.name = generate_unique_name_reactive(&duplicate.name)?;
    duplicate.created_at = chrono::Utc::now();

    // Add duplicate
    course_context.courses.write().push(duplicate);
    log::info!("Course {id} duplicated successfully");
    Ok(())
}

/// Structure a course with the given structure
pub fn structure_course_reactive(id: Uuid, structure: CourseStructure) -> StateResult<()> {
    let mut course_context = use_context::<CourseContext>();
    let mut courses = course_context.courses.write();

    let course_index = courses
        .iter()
        .position(|c| c.id == id)
        .ok_or(StateError::CourseNotFound(id))?;

    // Validation
    if structure.modules.is_empty() {
        return Err(StateError::ValidationError(
            "Course structure cannot be empty".to_string(),
        ));
    }

    // Atomic update
    courses[course_index].structure = Some(structure);
    log::info!("Course {id} structured successfully");
    Ok(())
}

/// Generate a unique course name
fn generate_unique_name_reactive(base_name: &str) -> StateResult<String> {
    let course_context = use_context::<CourseContext>();
    let courses = course_context.courses.read();
    let mut counter = 1;
    let mut new_name = format!("{base_name} (Copy)");

    // Keep incrementing until we find a unique name
    while courses.iter().any(|c| c.name == new_name) {
        counter += 1;
        new_name = format!("{base_name} (Copy {counter})");

        // Prevent infinite loops
        if counter > 1000 {
            return Err(StateError::ValidationError(
                "Could not generate unique course name".to_string(),
            ));
        }
    }

    Ok(new_name)
}

/// Start an import job
pub fn start_import_reactive(job: ImportJob) -> StateResult<()> {
    let mut import_context = use_context::<ImportContext>();
    import_context.active_import.set(Some(job));
    log::info!("Import started");
    Ok(())
}

/// Update import progress
pub fn update_import_reactive(id: Uuid, progress: f32, message: String) -> StateResult<()> {
    let mut import_context = use_context::<ImportContext>();
    let mut active_import = import_context.active_import.write();

    if let Some(ref mut import) = *active_import {
        if import.id == id {
            import.update_progress(progress, message);
            return Ok(());
        }
    }
    Err(StateError::InvalidOperation(
        "No active import found".to_string(),
    ))
}

/// Complete an import job
pub fn complete_import_reactive(id: Uuid) -> StateResult<()> {
    let mut import_context = use_context::<ImportContext>();
    let mut active_import = import_context.active_import.write();

    if let Some(ref mut import) = *active_import {
        if import.id == id {
            import.status = ImportStatus::Completed;
            return Ok(());
        }
    }
    Err(StateError::InvalidOperation(
        "No active import found".to_string(),
    ))
}

/// Fail an import job
pub fn fail_import_reactive(id: Uuid, error: String) -> StateResult<()> {
    let mut import_context = use_context::<ImportContext>();
    let mut active_import = import_context.active_import.write();

    if let Some(ref mut import) = *active_import {
        if import.id == id {
            import.mark_failed(error);
            return Ok(());
        }
    }
    Err(StateError::InvalidOperation(
        "No active import found".to_string(),
    ))
}

/// Clear the active import
pub fn clear_import_reactive() -> StateResult<()> {
    let mut import_context = use_context::<ImportContext>();
    import_context.active_import.set(None);
    log::info!("Import cleared");
    Ok(())
}

/// Set video context for the contextual panel and open notes tab
pub fn set_video_context_and_open_notes_reactive(video_context: VideoContext) -> StateResult<()> {
    let mut panel_context = use_context::<ContextualPanelContext>();
    let mut state = panel_context.state.write();

    state.video_context = Some(video_context);
    state.active_tab = ContextualPanelTab::Notes;
    state.is_open = true;

    log::info!("Video context set and notes panel opened");
    Ok(())
}

/// Clear video context from the contextual panel
pub fn clear_video_context_reactive() -> StateResult<()> {
    let mut panel_context = use_context::<ContextualPanelContext>();
    panel_context.state.write().video_context = None;
    log::info!("Video context cleared");
    Ok(())
}

/// Toggle mobile sidebar
pub fn toggle_mobile_sidebar_reactive() -> StateResult<()> {
    let mut sidebar_context = use_context::<MobileSidebarContext>();
    let current = *sidebar_context.is_open.read();
    sidebar_context.is_open.set(!current);
    Ok(())
}

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

/// Initialize global state from legacy AppState
pub fn initialize_global_state(app_state: Signal<AppState>) {
    let mut course_context = use_context::<CourseContext>();
    let mut notes_context = use_context::<NotesContext>();
    let mut plan_context = use_context::<PlanContext>();
    let mut import_context = use_context::<ImportContext>();
    let mut panel_context = use_context::<ContextualPanelContext>();
    let mut sidebar_context = use_context::<MobileSidebarContext>();

    // Sync initial state in an effect to avoid render-time signal writes
    use_effect(move || {
        let state = app_state.read();
        course_context.courses.set(state.courses.clone());
        notes_context.notes.set(state.notes.clone());
        plan_context.plans.set(state.plans.clone());
        import_context
            .active_import
            .set(state.active_import.clone());
        panel_context.state.set(state.contextual_panel.clone());
        sidebar_context.is_open.set(state.sidebar_open_mobile);
    });
}

// ============================================================================
// LEGACY FUNCTIONS (for backward compatibility)
// ============================================================================

/// Legacy function - use add_course_reactive instead
pub fn add_course(mut app_state: Signal<AppState>, course: Course) -> StateResult<()> {
    // Validation
    if course.name.trim().is_empty() {
        return Err(StateError::ValidationError(
            "Course name cannot be empty".to_string(),
        ));
    }

    // Check for duplicate names
    {
        let state = app_state.read();
        if state.courses.iter().any(|c| c.name == course.name) {
            return Err(StateError::ValidationError(
                "Course with this name already exists".to_string(),
            ));
        }
    }

    // Atomic update
    app_state.write().courses.push(course);
    log::info!("Course added successfully");
    Ok(())
}

/// Legacy function - use update_course_reactive instead
pub fn update_course(
    mut app_state: Signal<AppState>,
    id: Uuid,
    updated_course: Course,
) -> StateResult<()> {
    let mut state = app_state.write();
    let course_index = state
        .courses
        .iter()
        .position(|c| c.id == id)
        .ok_or(StateError::CourseNotFound(id))?;

    // Validation
    if updated_course.name.trim().is_empty() {
        return Err(StateError::ValidationError(
            "Course name cannot be empty".to_string(),
        ));
    }

    // Check for duplicate names (excluding current course)
    if state
        .courses
        .iter()
        .any(|c| c.id != id && c.name == updated_course.name)
    {
        return Err(StateError::ValidationError(
            "Course with this name already exists".to_string(),
        ));
    }

    // Atomic update
    state.courses[course_index] = updated_course;
    log::info!("Course {id} updated successfully");
    Ok(())
}

/// Legacy function - use delete_course_reactive instead
pub fn delete_course(mut app_state: Signal<AppState>, id: Uuid) -> StateResult<()> {
    let mut state = app_state.write();
    let initial_len = state.courses.len();

    // Atomic update
    state.courses.retain(|c| c.id != id);

    if state.courses.len() == initial_len {
        return Err(StateError::CourseNotFound(id));
    }

    log::info!("Course {id} deleted successfully");
    Ok(())
}

/// Legacy function - use set_video_context_and_open_notes_reactive instead
pub fn set_video_context_and_open_notes(
    mut app_state: Signal<AppState>,
    video_context: VideoContext,
) -> StateResult<()> {
    let mut state = app_state.write();
    state.contextual_panel.video_context = Some(video_context);
    state.contextual_panel.active_tab = ContextualPanelTab::Notes;
    state.contextual_panel.is_open = true;
    log::info!("Video context set and notes panel opened");
    Ok(())
}

// ============================================================================
// REACTIVE HOOKS FOR LEGACY COMPATIBILITY
// ============================================================================

/// Hook for reactive access to courses (legacy compatibility)
pub fn use_courses() -> Memo<Vec<Course>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || app_state.read().courses.clone())
}

/// Hook for reactive access to a specific course (legacy compatibility)
pub fn use_course(id: Uuid) -> Memo<Option<Course>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || {
        app_state
            .read()
            .courses
            .iter()
            .find(|c| c.id == id)
            .cloned()
    })
}

/// Hook for reactive access to active import (legacy compatibility)
pub fn use_active_import() -> Memo<Option<ImportJob>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || app_state.read().active_import.clone())
}

/// Hook for course statistics (legacy compatibility)
pub fn use_course_stats() -> Memo<(usize, usize, usize)> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || {
        let courses = &app_state.read().courses;
        let total = courses.len();
        let structured = courses.iter().filter(|c| c.is_structured()).count();
        let videos = courses.iter().map(|c| c.video_count()).sum();
        (total, structured, videos)
    })
}

/// Hook for getting tag statistics from notes (legacy compatibility)
pub fn use_tag_statistics() -> Memo<std::collections::HashMap<String, usize>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || {
        let mut stats = std::collections::HashMap::new();
        for note in &app_state.read().notes {
            for tag in &note.tags {
                *stats.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        stats
    })
}

// ============================================================================
// ASYNC OPERATIONS
// ============================================================================

/// Async-safe course structuring
pub async fn async_structure_course(course_id: Uuid, raw_titles: Vec<String>) -> StateResult<()> {
    // Structure the course (this should be an async operation)
    match crate::nlp::structure_course(raw_titles) {
        Ok(structure) => {
            structure_course_reactive(course_id, structure)?;
            Ok(())
        }
        Err(e) => Err(StateError::InvalidOperation(format!(
            "Failed to structure course: {e}"
        ))),
    }
}

// ============================================================================
// READ-ONLY HELPER FUNCTIONS
// ============================================================================

/// Check if a course exists
pub fn course_exists_reactive(id: Uuid) -> bool {
    let course_context = use_context::<CourseContext>();
    course_context.courses.read().iter().any(|c| c.id == id)
}

/// Get a course by ID
pub fn get_course_reactive(id: Uuid) -> Option<Course> {
    let course_context = use_context::<CourseContext>();
    course_context
        .courses
        .read()
        .iter()
        .find(|c| c.id == id)
        .cloned()
}

/// Get all courses
pub fn get_courses_reactive() -> Vec<Course> {
    let course_context = use_context::<CourseContext>();
    course_context.courses.read().clone()
}

/// Get active import
pub fn get_active_import_reactive() -> Option<ImportJob> {
    let import_context = use_context::<ImportContext>();
    import_context.active_import.read().clone()
}

/// Get course statistics
pub fn get_course_stats_reactive() -> (usize, usize, usize) {
    let course_context = use_context::<CourseContext>();
    let courses = &course_context.courses.read();
    let total = courses.len();
    let structured = courses.iter().filter(|c| c.is_structured()).count();
    let videos = courses.iter().map(|c| c.video_count()).sum();
    (total, structured, videos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Course;
    use chrono::Utc;

    #[allow(dead_code)] // Helper function for tests
    fn create_test_course(name: &str) -> Course {
        Course {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Lesson 1".to_string()],
            structure: None,
        }
    }

    #[test]
    fn test_course_context_creation() {
        let context = CourseContext::new();
        assert_eq!(context.courses.read().len(), 0);
    }

    #[test]
    fn test_import_context_creation() {
        let context = ImportContext::new();
        assert!(context.active_import.read().is_none());
    }

    #[test]
    fn test_contextual_panel_context_creation() {
        let context = ContextualPanelContext::new();
        assert!(!context.state.read().is_open);
    }

    #[test]
    fn test_mobile_sidebar_context_creation() {
        let context = MobileSidebarContext::new();
        assert!(!context.is_open.read().clone());
    }
}
