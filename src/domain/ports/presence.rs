//! Presence Port - Interface for external activity reporting (e.g., Discord Rich Presence).

/// Represents the current user activity within the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Activity {
    /// The application is open but the user is not in a specific section.
    Idle,
    /// User is viewing the main dashboard.
    Dashboard,
    /// User is browsing the course library.
    BrowsingCourses,
    /// User is watching a specific video in a course.
    Watching { course_title: String, video_title: String },
    /// User is taking an exam for a video/course.
    TakingExam { course_title: String, exam_title: String },
    /// User is in the settings menu.
    Settings,
}

/// Port for managing external presence/status.
pub trait PresenceProvider: Send + Sync {
    /// Updates the current activity displayed on external platforms.
    fn update_activity(&self, activity: Activity);

    /// Clears the current activity.
    fn clear_activity(&self);

    /// Returns whether the presence provider is currently connected.
    fn is_connected(&self) -> bool;
}
