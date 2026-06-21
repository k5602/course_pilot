//! Page trait and page module declarations.

use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;

pub mod course_list;
pub mod course_view;
pub mod dashboard;
pub mod onboarding;
pub mod quiz_list;
pub mod quiz_view;
pub mod settings;
pub mod video_player;

/// Common contract for all navigable pages in the application.
///
/// Every page must provide its root widget and support refresh on navigation.
/// Optional methods (`stop`, `set_nav_pages`) have default no-op implementations
/// for pages that don't need them.
pub trait Page {
    /// Returns the root widget for this page.
    fn widget(&self) -> &gtk::Widget;

    /// Refreshes the page content. Called when the page becomes visible.
    fn refresh(&self);

    /// Called when navigating away from this page. Default is no-op.
    /// Only needed for pages with active resources (e.g., video playback).
    fn stop(&self) {}

    /// Injects the navigation page map for sub-page navigation.
    /// Default is no-op for pages that never navigate to sub-pages.
    fn set_nav_pages(&self, _pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {}
}

impl Page for dashboard::DashboardPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }

    fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {
        self.set_nav_pages(pages);
    }
}

impl Page for course_list::CourseListPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }

    fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {
        self.set_nav_pages(pages);
    }
}

impl Page for course_view::CourseViewPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }

    fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {
        self.set_nav_pages(pages);
    }
}

impl Page for video_player::VideoPlayerPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }

    fn stop(&self) {
        self.stop();
    }

    fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {
        self.set_nav_pages(pages);
    }
}

impl Page for quiz_list::QuizListPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }

    fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {
        self.set_nav_pages(pages);
    }
}

impl Page for quiz_view::QuizViewPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }
}

impl Page for settings::SettingsPage {
    fn widget(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }

    fn refresh(&self) {
        self.refresh();
    }
}
