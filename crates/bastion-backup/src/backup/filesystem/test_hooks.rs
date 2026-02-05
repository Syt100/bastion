use std::cell::RefCell;
use std::path::Path;

thread_local! {
    static AFTER_FILE_OPEN: RefCell<Option<Box<dyn Fn(&Path, &str)>>> = RefCell::new(None);
}

pub(super) fn set_after_file_open_hook(hook: Option<Box<dyn Fn(&Path, &str)>>) {
    AFTER_FILE_OPEN.with(|slot| {
        *slot.borrow_mut() = hook;
    });
}

pub(super) fn run_after_file_open_hook(fs_path: &Path, archive_path: &str) {
    AFTER_FILE_OPEN.with(|slot| {
        if let Some(cb) = slot.borrow_mut().as_mut() {
            cb(fs_path, archive_path);
        }
    });
}
