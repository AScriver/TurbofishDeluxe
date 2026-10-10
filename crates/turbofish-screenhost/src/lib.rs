//! Windows screensaver switch and preview-window host primitives.
//!
//! This crate does not register a screensaver, create a game window, or touch
//! game saves. The caller owns window creation, rendering, input dismissal,
//! and quitting when a preview parent closes.

use std::{fmt, num::NonZeroUsize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenCommand {
    Fullscreen,
    Preview(ParentWindow),
    Configure(Option<ParentWindow>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParentWindow(NonZeroUsize);

impl ParentWindow {
    pub fn decimal_handle(self) -> usize {
        self.0.get()
    }

    /// Check existence before opening a preview or a parented preferences UI.
    /// The preview attachment checks again because the host can close meanwhile.
    pub fn validate(self) -> Result<(), HostError> {
        platform::validate_parent(self).map(|_| ())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewStatus {
    Alive,
    ParentClosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostError {
    InvalidCommand,
    MissingHandle,
    InvalidHandle,
    UnsupportedPlatform,
    NoActiveWindow,
    WindowNotOwned,
    ParentClosed,
    ChildClosed,
    WindowDetached,
    EmptyParentClient,
    WindowsApi(&'static str, u32),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCommand => write!(f, "unsupported screensaver command line"),
            Self::MissingHandle => write!(f, "screensaver parent HWND is missing"),
            Self::InvalidHandle => write!(
                f,
                "screensaver parent HWND must be a nonzero decimal integer"
            ),
            Self::UnsupportedPlatform => write!(f, "native screensaver preview requires Windows"),
            Self::NoActiveWindow => write!(f, "no active window on the calling UI thread"),
            Self::WindowNotOwned => write!(f, "window is not owned by this process and UI thread"),
            Self::ParentClosed => write!(f, "screensaver preview parent is closed"),
            Self::ChildClosed => write!(f, "project preview window is closed"),
            Self::WindowDetached => write!(f, "project preview window left its parent"),
            Self::EmptyParentClient => write!(f, "screensaver preview parent has no client area"),
            Self::WindowsApi(name, code) => write!(f, "{name} failed with Win32 error {code}"),
        }
    }
}

impl std::error::Error for HostError {}

/// Parse arguments after the executable path. No unknown switch, malformed
/// HWND, or missing `/p` handle falls back to gameplay.
pub fn parse_command(args: &[String]) -> Result<ScreenCommand, HostError> {
    let Some(first) = args.first() else {
        return Err(HostError::InvalidCommand);
    };
    let (switch, inline) = first
        .split_once(':')
        .map_or((first.as_str(), None), |(a, b)| (a, Some(b)));
    if switch.eq_ignore_ascii_case("/s") && inline.is_none() && args.len() == 1 {
        return Ok(ScreenCommand::Fullscreen);
    }
    if switch.eq_ignore_ascii_case("/p") {
        let raw = match (inline, args.get(1).map(String::as_str), args.len()) {
            (Some(raw), None, 1) | (None, Some(raw), 2) => raw,
            (None, None, 1) => return Err(HostError::MissingHandle),
            _ => return Err(HostError::InvalidCommand),
        };
        return Ok(ScreenCommand::Preview(parse_handle(raw)?));
    }
    if switch.eq_ignore_ascii_case("/c") {
        return match (inline, args.get(1).map(String::as_str), args.len()) {
            (None, None, 1) => Ok(ScreenCommand::Configure(None)),
            (Some(raw), None, 1) | (None, Some(raw), 2) => {
                Ok(ScreenCommand::Configure(Some(parse_handle(raw)?)))
            }
            _ => Err(HostError::InvalidCommand),
        };
    }
    Err(HostError::InvalidCommand)
}

fn parse_handle(raw: &str) -> Result<ParentWindow, HostError> {
    if raw.is_empty() {
        return Err(HostError::MissingHandle);
    }
    if !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(HostError::InvalidHandle);
    }
    let value = raw.parse::<usize>().map_err(|_| HostError::InvalidHandle)?;
    NonZeroUsize::new(value)
        .map(ParentWindow)
        .ok_or(HostError::InvalidHandle)
}

pub use platform::{MonitorPowerGuard, OwnWindow, PreviewAttachment};

#[cfg(windows)]
mod platform {
    use super::{HostError, ParentWindow, PreviewStatus};
    use std::{marker::PhantomData, num::NonZeroUsize, rc::Rc};
    use windows_sys::Win32::{
        Foundation::{GetLastError, HWND, RECT, SetLastError},
        System::{
            Power::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED, SetThreadExecutionState},
            Threading::{GetCurrentProcessId, GetCurrentThreadId},
        },
        UI::{
            Input::KeyboardAndMouse::GetActiveWindow,
            WindowsAndMessaging::{
                GA_ROOT, GWL_STYLE, GetAncestor, GetClientRect, GetParent, GetWindowLongPtrW,
                GetWindowThreadProcessId, IsWindow, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOZORDER,
                SetParent, SetWindowLongPtrW, SetWindowPos, WS_BORDER, WS_CAPTION, WS_CHILD,
                WS_DLGFRAME, WS_POPUP, WS_THICKFRAME,
            },
        },
    };

    #[derive(Clone, Copy)]
    struct Identity {
        thread: u32,
        process: u32,
    }

    fn hwnd(raw: NonZeroUsize) -> HWND {
        raw.get() as HWND
    }

    fn identity(window: HWND) -> Option<Identity> {
        if window.is_null() || unsafe { IsWindow(window) } == 0 {
            return None;
        }
        let mut process = 0;
        let thread = unsafe { GetWindowThreadProcessId(window, &mut process) };
        (thread != 0 && process != 0).then_some(Identity { thread, process })
    }

    fn own_window(window: HWND) -> bool {
        identity(window).is_some_and(|id| {
            id.thread == unsafe { GetCurrentThreadId() }
                && id.process == unsafe { GetCurrentProcessId() }
        })
    }

    fn parent_info(parent: ParentWindow) -> Result<(Identity, i32, i32), HostError> {
        let window = hwnd(parent.0);
        let id = identity(window).ok_or(HostError::ParentClosed)?;
        let mut rect = RECT::default();
        if unsafe { GetClientRect(window, &mut rect) } == 0 {
            return Err(last_error("GetClientRect"));
        }
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        if width <= 0 || height <= 0 {
            return Err(HostError::EmptyParentClient);
        }
        Ok((id, width, height))
    }

    pub(super) fn validate_parent(parent: ParentWindow) -> Result<(), HostError> {
        parent_info(parent).map(|_| ())
    }

    fn last_error(name: &'static str) -> HostError {
        HostError::WindowsApi(name, unsafe { GetLastError() })
    }

    fn style(window: HWND) -> Result<isize, HostError> {
        unsafe { SetLastError(0) };
        let value = unsafe { GetWindowLongPtrW(window, GWL_STYLE) };
        if value == 0 && unsafe { GetLastError() } != 0 {
            Err(last_error("GetWindowLongPtrW"))
        } else {
            Ok(value)
        }
    }

    fn set_style(window: HWND, value: isize) -> Result<(), HostError> {
        unsafe { SetLastError(0) };
        let old = unsafe { SetWindowLongPtrW(window, GWL_STYLE, value) };
        if old == 0 && unsafe { GetLastError() } != 0 {
            Err(last_error("SetWindowLongPtrW"))
        } else {
            Ok(())
        }
    }

    fn set_parent(child: HWND, parent: HWND) -> Result<HWND, HostError> {
        unsafe { SetLastError(0) };
        let old = unsafe { SetParent(child, parent) };
        if old.is_null() && unsafe { GetLastError() } != 0 {
            Err(last_error("SetParent"))
        } else {
            Ok(old)
        }
    }

    fn resize(child: HWND, width: i32, height: i32) -> Result<(), HostError> {
        if unsafe {
            SetWindowPos(
                child,
                std::ptr::null_mut(),
                0,
                0,
                width,
                height,
                SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOZORDER,
            )
        } == 0
        {
            Err(last_error("SetWindowPos"))
        } else {
            Ok(())
        }
    }

    /// Thread-local display-idle policy for a running saver. `power_save`
    /// leaves Windows monitor policy untouched; false holds the display on.
    /// Keep this guard on the same UI thread and drop it before that thread
    /// exits. Other execution-state owners on this thread must nest LIFO.
    pub struct MonitorPowerGuard {
        previous: Option<u32>,
        _same_thread: PhantomData<Rc<()>>,
    }

    impl MonitorPowerGuard {
        pub fn new(power_save: bool) -> Result<Self, HostError> {
            if power_save {
                return Ok(Self {
                    previous: None,
                    _same_thread: PhantomData,
                });
            }
            let previous = unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED) };
            if previous == 0 {
                return Err(last_error("SetThreadExecutionState"));
            }
            // Preserve any prior system/away requirement from another owner
            // on this thread while adding only our display requirement.
            let combined = previous | ES_CONTINUOUS | ES_DISPLAY_REQUIRED;
            if combined != (ES_CONTINUOUS | ES_DISPLAY_REQUIRED)
                && unsafe { SetThreadExecutionState(combined) } == 0
            {
                let error = last_error("SetThreadExecutionState");
                let _ = unsafe { SetThreadExecutionState(previous | ES_CONTINUOUS) };
                return Err(error);
            }
            Ok(Self {
                previous: Some(previous),
                _same_thread: PhantomData,
            })
        }
    }

    impl Drop for MonitorPowerGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous {
                let _ = unsafe { SetThreadExecutionState(previous | ES_CONTINUOUS) };
            }
        }
    }

    /// Capture only the active window of this calling UI thread. miniquad
    /// 0.4.11 has no public Windows HWND getter; absence is an explicit error.
    pub struct OwnWindow {
        raw: NonZeroUsize,
        _same_thread: PhantomData<Rc<()>>,
    }

    impl OwnWindow {
        pub fn capture_active() -> Result<Self, HostError> {
            let window = unsafe { GetActiveWindow() };
            let raw = NonZeroUsize::new(window as usize).ok_or(HostError::NoActiveWindow)?;
            if !own_window(window) || unsafe { GetAncestor(window, GA_ROOT) } != window {
                return Err(HostError::WindowNotOwned);
            }
            Ok(Self {
                raw,
                _same_thread: PhantomData,
            })
        }

        /// Attach this project-owned top-level window to an external preview
        /// parent. Call on its UI thread after window creation.
        pub fn attach_preview(&self, parent: ParentWindow) -> Result<PreviewAttachment, HostError> {
            let child = hwnd(self.raw);
            if !own_window(child) || unsafe { GetAncestor(child, GA_ROOT) } != child {
                return Err(HostError::WindowNotOwned);
            }
            let (parent_id, width, height) = parent_info(parent)?;
            let parent_hwnd = hwnd(parent.0);
            if child == parent_hwnd {
                return Err(HostError::InvalidHandle);
            }
            let previous_style = style(child)?;
            let child_style = ((previous_style as u32 | WS_CHILD)
                & !(WS_POPUP | WS_BORDER | WS_CAPTION | WS_DLGFRAME | WS_THICKFRAME))
                as isize;
            set_style(child, child_style)?;
            let previous_parent = match set_parent(child, parent_hwnd) {
                Ok(old) => old,
                Err(error) => {
                    let _ = set_style(child, previous_style);
                    return Err(error);
                }
            };
            if let Err(error) = resize(child, width, height) {
                let _ = set_parent(child, previous_parent);
                let _ = set_style(child, previous_style);
                return Err(error);
            }
            Ok(PreviewAttachment {
                child: self.raw,
                parent,
                parent_id,
                previous_parent,
                previous_style,
                size: (width, height),
                _same_thread: PhantomData,
            })
        }
    }

    /// Owns temporary parenting. Drop restores it when child still exists.
    pub struct PreviewAttachment {
        child: NonZeroUsize,
        parent: ParentWindow,
        parent_id: Identity,
        previous_parent: HWND,
        previous_style: isize,
        size: (i32, i32),
        _same_thread: PhantomData<Rc<()>>,
    }

    impl PreviewAttachment {
        /// Poll from the child UI thread. Resize on parent changes; caller
        /// should stop the screensaver promptly on `ParentClosed`.
        pub fn poll(&mut self) -> Result<PreviewStatus, HostError> {
            let (id, width, height) = match parent_info(self.parent) {
                Ok(info) => info,
                Err(HostError::ParentClosed) => return Ok(PreviewStatus::ParentClosed),
                Err(error) => return Err(error),
            };
            if id.thread != self.parent_id.thread || id.process != self.parent_id.process {
                return Ok(PreviewStatus::ParentClosed);
            }
            let child = hwnd(self.child);
            if !own_window(child) {
                return Err(HostError::ChildClosed);
            }
            if unsafe { GetParent(child) } != hwnd(self.parent.0) {
                return Err(HostError::WindowDetached);
            }
            if (width, height) != self.size {
                resize(child, width, height)?;
                self.size = (width, height);
            }
            Ok(PreviewStatus::Alive)
        }
    }

    impl Drop for PreviewAttachment {
        fn drop(&mut self) {
            let child = hwnd(self.child);
            if own_window(child) && unsafe { GetParent(child) } == hwnd(self.parent.0) {
                let _ = set_parent(child, self.previous_parent);
                let _ = set_style(child, self.previous_style);
                let _ = unsafe {
                    SetWindowPos(
                        child,
                        std::ptr::null_mut(),
                        0,
                        0,
                        0,
                        0,
                        SWP_FRAMECHANGED
                            | SWP_NOACTIVATE
                            | SWP_NOZORDER
                            | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                            | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE,
                    )
                };
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{HostError, ParentWindow, PreviewStatus};

    pub(super) fn validate_parent(_parent: ParentWindow) -> Result<(), HostError> {
        Err(HostError::UnsupportedPlatform)
    }

    pub struct OwnWindow;
    impl OwnWindow {
        pub fn capture_active() -> Result<Self, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
        pub fn attach_preview(
            &self,
            _parent: ParentWindow,
        ) -> Result<PreviewAttachment, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
    }
    pub struct PreviewAttachment;
    impl PreviewAttachment {
        pub fn poll(&mut self) -> Result<PreviewStatus, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
    }
    pub struct MonitorPowerGuard;
    impl MonitorPowerGuard {
        pub fn new(_power_save: bool) -> Result<Self, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HostError, ScreenCommand, parse_command};

    #[test]
    fn preview_requires_only_one_nonzero_decimal_parent() {
        let args = ["/P:12345".to_owned()];
        assert!(
            matches!(parse_command(&args), Ok(ScreenCommand::Preview(parent)) if parent.decimal_handle() == 12345)
        );
        for args in [
            vec!["/p"],
            vec!["/p:0"],
            vec!["/p:0x123"],
            vec!["/p", "123", "extra"],
        ] {
            let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
            assert!(matches!(
                parse_command(&args),
                Err(HostError::MissingHandle
                    | HostError::InvalidHandle
                    | HostError::InvalidCommand)
            ));
        }
    }
}
