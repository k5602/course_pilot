//! Modern State Management Utilities
//!
//! This module provides reactive state patterns and utilities for Course Pilot
//! using modern dioxus-signals patterns.

use dioxus::prelude::*;

// Re-export the main state management functions
pub use crate::state::{
    ContextualPanelContext, ContextualPanelContextProvider, CourseContext, CourseContextProvider,
    ImportContext, ImportContextProvider, MobileSidebarContext, MobileSidebarContextProvider,
    NotesContext, NotesContextProvider, PlanContext, PlanContextProvider, initialize_global_state,
    use_active_import_reactive, use_contextual_panel_reactive, use_course_reactive,
    use_course_stats_reactive, use_courses_reactive, use_mobile_sidebar_reactive,
    use_notes_reactive, use_plans_reactive,
};

/// Utility hook for managing loading states across contexts
pub fn use_loading_state() -> (Signal<bool>, Callback<bool>) {
    let loading = use_signal(|| false);
    let set_loading = use_callback({
        let mut loading = loading;
        move |is_loading: bool| loading.set(is_loading)
    });
    (loading, set_loading)
}

/// Utility hook for managing error states across contexts
pub fn use_error_state() -> (Signal<Option<String>>, Callback<Option<String>>) {
    let error = use_signal(|| None);
    let set_error = use_callback({
        let mut error = error;
        move |err: Option<String>| error.set(err)
    });
    (error, set_error)
}

/// Hook for managing search state
pub fn use_search_state() -> (Signal<String>, Callback<String>) {
    let search_query = use_signal(String::new);
    let set_search_query = use_callback({
        let mut search_query = search_query;
        move |query: String| search_query.set(query)
    });
    (search_query, set_search_query)
}

/// Hook for managing pagination state
#[derive(Clone, Copy)]
pub struct PaginationState {
    pub current_page: Signal<usize>,
    pub items_per_page: Signal<usize>,
    pub total_items: Signal<usize>,
}

impl PaginationState {
    pub fn new(items_per_page: usize) -> Self {
        Self {
            current_page: Signal::new(1),
            items_per_page: Signal::new(items_per_page),
            total_items: Signal::new(0),
        }
    }

    pub fn total_pages(&self) -> usize {
        let total = (self.total_items)();
        let per_page = (self.items_per_page)();
        if per_page == 0 { 1 } else { total.div_ceil(per_page) }
    }

    pub fn has_next_page(&self) -> bool {
        (self.current_page)() < self.total_pages()
    }

    pub fn has_prev_page(&self) -> bool {
        (self.current_page)() > 1
    }
}

pub fn use_pagination_state(items_per_page: usize) -> PaginationState {
    use_signal(|| PaginationState::new(items_per_page))()
}

/// Hook for managing selection state (for bulk operations)
pub fn use_selection_state<T: Clone + PartialEq + 'static>()
-> (Signal<Vec<T>>, Callback<T>, Callback<T>, Callback<()>, Callback<Vec<T>>) {
    let selected_items = use_signal(|| Vec::new());

    let add_selection = use_callback({
        let mut selected_items = selected_items;
        move |item: T| {
            let mut items = selected_items.read().clone();
            if !items.contains(&item) {
                items.push(item);
                selected_items.set(items);
            }
        }
    });

    let remove_selection = use_callback({
        let mut selected_items = selected_items;
        move |item: T| {
            let mut items = selected_items.read().clone();
            items.retain(|i| i != &item);
            selected_items.set(items);
        }
    });

    let clear_selection = use_callback({
        let mut selected_items = selected_items;
        move |_| selected_items.set(Vec::new())
    });

    let set_selection = use_callback({
        let mut selected_items = selected_items;
        move |items: Vec<T>| selected_items.set(items)
    });

    (selected_items, add_selection, remove_selection, clear_selection, set_selection)
}

/// Hook for managing form validation state
pub fn use_validation_state() -> (
    Signal<std::collections::HashMap<String, String>>,
    Callback<(String, String)>,
    Callback<String>,
    Callback<()>,
) {
    let errors = use_signal(std::collections::HashMap::new);

    let set_error = use_callback({
        let mut errors = errors;
        move |(field, message): (String, String)| {
            let mut error_map = errors.read().clone();
            error_map.insert(field, message);
            errors.set(error_map);
        }
    });

    let clear_error = use_callback({
        let mut errors = errors;
        move |field: String| {
            let mut error_map = errors.read().clone();
            error_map.remove(&field);
            errors.set(error_map);
        }
    });

    let clear_all_errors = use_callback({
        let mut errors = errors;
        move |_| errors.set(std::collections::HashMap::new())
    });

    (errors, set_error, clear_error, clear_all_errors)
}

/// Utility for debounced state updates (useful for search)
pub fn use_debounced_state<T: Clone + PartialEq + 'static>(
    initial_value: T,
    delay_ms: u64,
) -> (Signal<T>, Signal<T>, Callback<T>) {
    let immediate_value = use_signal(|| initial_value.clone());
    let debounced_value = use_signal(|| initial_value);

    let set_value = use_callback({
        let mut immediate_value = immediate_value;
        let mut debounced_value = debounced_value;
        move |value: T| {
            immediate_value.set(value.clone());

            // Debounce the update
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                debounced_value.set(value);
            });
        }
    });

    (immediate_value, debounced_value, set_value)
}

/// Hook for managing async operation state
#[derive(Clone, Copy)]
pub struct AsyncOperationState<T: Clone + 'static> {
    pub data: Signal<Option<T>>,
    pub loading: Signal<bool>,
    pub error: Signal<Option<String>>,
}

impl<T: Clone + 'static> Default for AsyncOperationState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + 'static> AsyncOperationState<T> {
    pub fn new() -> Self {
        Self { data: Signal::new(None), loading: Signal::new(false), error: Signal::new(None) }
    }

    pub fn set_loading(&mut self, is_loading: bool) {
        self.loading.set(is_loading);
        if is_loading {
            self.error.set(None);
        }
    }

    pub fn set_data(&mut self, data: T) {
        self.data.set(Some(data));
        self.loading.set(false);
        self.error.set(None);
    }

    pub fn set_error(&mut self, error: String) {
        self.error.set(Some(error));
        self.loading.set(false);
    }
}

pub fn use_async_operation_state<T: Clone + 'static>() -> AsyncOperationState<T> {
    use_signal(|| AsyncOperationState::new())()
}
