//! Windows crash suppression for GStreamer shutdown.
//!
//! GStreamer on Windows fires a STATUS_ACCESS_VIOLATION (0xc0000005) during
//! pipeline teardown when `set_state(Null)` is called. This happens inside
//! iced's `run()`, so any cleanup after `run()` is never reached.
//!
//! The fix: install a Windows unhandled-exception filter that calls
//! `TerminateProcess(GetCurrentProcess(), 0)` when a crash occurs after
//! the app has started closing.  Outside the close sequence, the filter is
//! a no-op so real crashes are still reported normally.

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, Ordering};

    static IS_CLOSING: AtomicBool = AtomicBool::new(false);

    extern "system" {
        fn SetUnhandledExceptionFilter(
            filter: Option<unsafe extern "system" fn(*mut c_void) -> i32>,
        ) -> Option<unsafe extern "system" fn(*mut c_void) -> i32>;
        fn TerminateProcess(h_process: isize, u_exit_code: u32) -> i32;
        fn GetCurrentProcess() -> isize;
    }

    pub fn mark_closing() {
        IS_CLOSING.store(true, Ordering::Release);
    }

    unsafe extern "system" fn exception_filter(_info: *mut c_void) -> i32 {
        if IS_CLOSING.load(Ordering::Acquire) {
            // Suppress the GStreamer shutdown crash — exit with code 0.
            unsafe {
                TerminateProcess(GetCurrentProcess(), 0);
            }
        }
        0 // EXCEPTION_CONTINUE_SEARCH — let other handlers deal with it
    }

    pub fn install() {
        // Install AFTER gstreamer::init() so we are the last filter set.
        unsafe {
            SetUnhandledExceptionFilter(Some(exception_filter));
        }
    }
}

/// Signal that the app is closing.
/// Any access violation that occurs after this call will be suppressed.
pub fn mark_closing() {
    #[cfg(windows)]
    imp::mark_closing();
}

/// Install the exception filter.  Call once, after all libraries are
/// initialised.  No-op on non-Windows platforms.
pub fn install() {
    #[cfg(windows)]
    imp::install();
}
