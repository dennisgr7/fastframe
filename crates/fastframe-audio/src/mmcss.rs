//! Windows: has the stream's thread join the Multimedia Class Scheduler
//! Service as an "Audio" task.
//!
//! cpal's WASAPI thread otherwise runs at normal priority, the same as any
//! background work, and Windows' power throttling (EcoQoS, or the Task
//! Manager's efficiency mode) lowers it further. MMCSS keeps the thread that
//! feeds the device ahead of that work, which is what Microsoft asks of the
//! thread rendering a WASAPI stream.
//! <https://learn.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service>
#![allow(unsafe_code)]

use std::cell::Cell;

thread_local! {
    /// Whether this thread has asked to join, so each stream thread asks once.
    static JOINED: Cell<bool> = const { Cell::new(false) };
}

/// Joins MMCSS as an "Audio" task, once per thread. A refusal is logged and
/// the stream plays on at normal priority. The task ends with the thread.
pub(crate) fn join_once() {
    JOINED.with(|joined| {
        if joined.replace(true) {
            return;
        }
        let task: Vec<u16> = "Audio".encode_utf16().chain([0]).collect();
        let mut index = 0u32;
        // SAFETY: `task` is a NUL-terminated UTF-16 string that outlives the
        // call, and `index` is a valid place for the task index.
        let handle = unsafe {
            windows_sys::Win32::System::Threading::AvSetMmThreadCharacteristicsW(
                task.as_ptr(),
                &mut index,
            )
        };
        if handle.is_null() {
            log::debug!(
                "audio output: the stream thread could not join MMCSS: {}",
                std::io::Error::last_os_error()
            );
        }
    });
}
