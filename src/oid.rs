use std::sync::atomic::{AtomicUsize, Ordering};

static mut NEXT: AtomicUsize = AtomicUsize::new(0);

pub fn oid() -> usize {
    unsafe { NEXT.fetch_add(1, Ordering::Relaxed) }
}
