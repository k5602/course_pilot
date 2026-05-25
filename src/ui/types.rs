use std::cell::RefCell;
use std::rc::Rc;

/// A late-binding refresh callback. Set after construction via `set_refresh_cb`
/// and called when the page needs to reload its content.
pub type RefreshCallback = Rc<RefCell<Option<Rc<dyn Fn()>>>>;
