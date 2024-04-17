use gg_platform_desktop::thread::DesktopThread;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

mod tests {
    use super::*;

    #[test]
    fn test_spawn_executes() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let thread = DesktopThread::new();
        thread.spawn(Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_available_parallelism() {
        let thread = DesktopThread::new();
        assert!(thread.available_parallelism() >= 1);
    }

    #[test]
    fn test_current_id() {
        let thread = DesktopThread::new();
        assert!(thread.current_id() < 1000);
    }
}
