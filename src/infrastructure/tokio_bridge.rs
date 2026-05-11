use std::sync::OnceLock;
use tokio::runtime::Runtime;

fn runtime() -> Option<&'static Runtime> {
    static RUNTIME: OnceLock<Option<Runtime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().ok()).as_ref()
}

pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    if let Some(rt) = runtime() {
        rt.spawn(future);
    } else {
        log::error!("Tokio runtime not available, cannot spawn async task");
    }
}

pub fn spawn_blocking<F, T>(f: F) -> Option<tokio::task::JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    runtime().map(|rt| rt.spawn_blocking(f))
}
