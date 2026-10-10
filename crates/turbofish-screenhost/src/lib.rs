//! Windows screensaver switch, preview and configuration-window host primitives.
//!
//! This crate does not register a screensaver, create a game window, or touch
//! game saves. The caller owns window creation, rendering, input dismissal,
//! and quitting when a preview parent or configuration owner closes.

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
    /// Attachments check again because the host can close meanwhile.
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
pub enum ConfigurationStatus {
    Alive,
    ParentClosed,
    WindowClosed,
    Released,
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
    InvalidOwner,
    OwnerChanged,
    PreviousOwnerClosed,
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
            Self::InvalidOwner => write!(
                f,
                "configuration owner must be a distinct top-level window without an ownership cycle"
            ),
            Self::OwnerChanged => write!(f, "project configuration window ownership changed"),
            Self::PreviousOwnerClosed => write!(
                f,
                "original configuration owner is no longer safe to restore"
            ),
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

pub use platform::{ConfigurationOwner, MonitorPowerGuard, OwnWindow, PreviewAttachment};

#[cfg(windows)]
mod platform {
    use super::{ConfigurationStatus, HostError, ParentWindow, PreviewStatus};
    use std::{io::Write, marker::PhantomData, num::NonZeroUsize, rc::Rc};
    use windows_sys::Win32::{
        Foundation::{GetLastError, HWND, RECT, SetLastError},
        System::{
            Power::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED, SetThreadExecutionState},
            Threading::{GetCurrentProcessId, GetCurrentThreadId},
        },
        UI::{
            Input::KeyboardAndMouse::GetActiveWindow,
            WindowsAndMessaging::{
                GA_ROOT, GWL_STYLE, GWLP_HWNDPARENT, GetAncestor, GetClientRect,
                GetForegroundWindow, GetParent, GetWindowLongPtrW, GetWindowThreadProcessId,
                IsWindow, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOZORDER, SetParent,
                SetWindowLongPtrW, SetWindowPos, WS_BORDER, WS_CAPTION, WS_CHILD, WS_DLGFRAME,
                WS_POPUP, WS_THICKFRAME,
            },
        },
    };

    #[derive(Clone, Copy, PartialEq, Eq)]
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

    fn owner(window: HWND) -> Result<HWND, HostError> {
        unsafe { SetLastError(0) };
        let value = unsafe { GetWindowLongPtrW(window, GWLP_HWNDPARENT) };
        if value == 0 && unsafe { GetLastError() } != 0 {
            Err(last_error("GetWindowLongPtrW(GWLP_HWNDPARENT)"))
        } else {
            Ok(value as HWND)
        }
    }

    fn set_owner(window: HWND, value: HWND) -> Result<HWND, HostError> {
        // An unowned window returns zero on success. Clear the thread's last
        // error before the call, then distinguish that from a native failure.
        unsafe { SetLastError(0) };
        let previous = unsafe { SetWindowLongPtrW(window, GWLP_HWNDPARENT, value as isize) };
        if previous == 0 && unsafe { GetLastError() } != 0 {
            Err(last_error("SetWindowLongPtrW(GWLP_HWNDPARENT)"))
        } else {
            Ok(previous as HWND)
        }
    }

    fn top_level(window: HWND) -> Result<bool, HostError> {
        Ok(unsafe { GetAncestor(window, GA_ROOT) } == window
            && style(window)? as u32 & WS_CHILD == 0)
    }

    fn valid_owner_chain(candidate: HWND, window: HWND) -> Result<bool, HostError> {
        // Reject a self-owner or an owner chain through this project window.
        // Read foreign window attributes only; never change them.
        let mut visited = Vec::new();
        let mut ancestor = candidate;
        while !ancestor.is_null() {
            if ancestor == window || visited.contains(&ancestor) {
                return Ok(false);
            }
            visited.push(ancestor);
            ancestor = owner(ancestor)?;
        }
        Ok(true)
    }

    fn configuration_parent(parent: ParentWindow, window: HWND) -> Result<Identity, HostError> {
        let (id, _, _) = parent_info(parent)?;
        let parent_hwnd = hwnd(parent.0);
        if !top_level(parent_hwnd)? || !valid_owner_chain(parent_hwnd, window)? {
            return Err(HostError::InvalidOwner);
        }
        Ok(id)
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

        /// Read focus only for the captured, still-owned window. No activation
        /// or input is sent to this or any external process.
        pub fn has_focus(&self) -> bool {
            let window = hwnd(self.raw);
            own_window(window) && unsafe { GetForegroundWindow() } == window
        }

        /// Set a supplied top-level window as this configuration window's
        /// temporary owner. Keep the guard on this UI thread until before
        /// destroying or recreating the project window; nest ownership guards
        /// LIFO. A missing parent needs no guard. This does not embed, resize,
        /// restyle, activate or change the supplied owner's window.
        pub fn attach_configuration(
            &self,
            parent: ParentWindow,
        ) -> Result<ConfigurationOwner, HostError> {
            let window = hwnd(self.raw);
            if !own_window(window) || !top_level(window)? {
                return Err(HostError::WindowNotOwned);
            }
            let window_id = identity(window).ok_or(HostError::WindowNotOwned)?;
            let parent_id = configuration_parent(parent, window)?;
            let previous_owner = owner(window)?;
            let previous_owner_id = if previous_owner.is_null() {
                None
            } else {
                Some(identity(previous_owner).ok_or(HostError::PreviousOwnerClosed)?)
            };
            // Recheck both identities immediately before changing only our
            // window. External-process lifetime checks cannot be atomic.
            if !own_window(window) || identity(window) != Some(window_id) || !top_level(window)? {
                return Err(HostError::WindowNotOwned);
            }
            if identity(hwnd(parent.0)) != Some(parent_id) {
                return Err(HostError::ParentClosed);
            }
            if !top_level(hwnd(parent.0))? || !valid_owner_chain(hwnd(parent.0), window)? {
                return Err(HostError::InvalidOwner);
            }
            let replaced_owner = set_owner(window, hwnd(parent.0))?;
            let guard = ConfigurationOwner {
                window: self.raw,
                window_id,
                parent,
                parent_id,
                previous_owner: replaced_owner,
                previous_owner_id: (replaced_owner == previous_owner)
                    .then_some(previous_owner_id)
                    .flatten(),
                attached: true,
                cleanup_error: None,
                _same_thread: PhantomData,
            };
            if replaced_owner != previous_owner {
                // A concurrent ownership change invalidated the old snapshot.
                // Drop detaches safely rather than restoring an unchecked HWND.
                return Err(HostError::OwnerChanged);
            }
            match guard.poll()? {
                ConfigurationStatus::Alive => Ok(guard),
                ConfigurationStatus::ParentClosed => Err(HostError::ParentClosed),
                ConfigurationStatus::WindowClosed | ConfigurationStatus::Released => {
                    Err(HostError::WindowNotOwned)
                }
            }
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

    /// Owns only the temporary owner attribute of the captured project window.
    /// Windows can destroy it when its external owner closes. Poll from its UI
    /// thread and let the caller close; no foreign process/window is controlled.
    /// HWND plus thread/process identity cannot distinguish handle reuse within
    /// the same thread/process. The caller must preserve its project window's
    /// lifetime; external-window checks and mutation cannot be atomic.
    #[must_use = "retain the guard until configuration closes"]
    pub struct ConfigurationOwner {
        window: NonZeroUsize,
        window_id: Identity,
        parent: ParentWindow,
        parent_id: Identity,
        previous_owner: HWND,
        previous_owner_id: Option<Identity>,
        attached: bool,
        cleanup_error: Option<HostError>,
        _same_thread: PhantomData<Rc<()>>,
    }

    impl ConfigurationOwner {
        pub fn poll(&self) -> Result<ConfigurationStatus, HostError> {
            if !self.attached {
                return Ok(ConfigurationStatus::Released);
            }
            let window = hwnd(self.window);
            if !own_window(window) || identity(window) != Some(self.window_id) {
                return Ok(ConfigurationStatus::WindowClosed);
            }
            if identity(hwnd(self.parent.0)) != Some(self.parent_id) {
                return Ok(ConfigurationStatus::ParentClosed);
            }
            if !top_level(hwnd(self.parent.0))? {
                return Ok(ConfigurationStatus::ParentClosed);
            }
            if !top_level(window)? || owner(window)? != hwnd(self.parent.0) {
                return Err(HostError::OwnerChanged);
            }
            Ok(ConfigurationStatus::Alive)
        }

        /// Restore before destroying the UI window so the caller can retain
        /// the outcome. A vanished project window needs no native mutation.
        /// A dead/replaced original owner is never reused: detach to no owner
        /// and return `PreviousOwnerClosed`. Changed ownership is left alone.
        /// Native failures leave cleanup armed for a later attempt or Drop.
        /// Successful restoration reports `ParentClosed` if the supplied owner
        /// disappeared meanwhile, otherwise `Released`.
        pub fn restore(&mut self) -> Result<ConfigurationStatus, HostError> {
            let result = self.restore_inner();
            if let Err(error) = result {
                self.cleanup_error = Some(error);
            }
            result
        }

        /// Latest cleanup failure remains available even after a successful
        /// retry. Drop also reports retained failures to standard error.
        pub fn cleanup_error(&self) -> Option<HostError> {
            self.cleanup_error
        }

        fn restore_inner(&mut self) -> Result<ConfigurationStatus, HostError> {
            if !self.attached {
                return Ok(ConfigurationStatus::Released);
            }
            let window = hwnd(self.window);
            if !own_window(window) || identity(window) != Some(self.window_id) {
                self.attached = false;
                return Ok(ConfigurationStatus::WindowClosed);
            }
            if !top_level(window)? || owner(window)? != hwnd(self.parent.0) {
                self.attached = false;
                return Err(HostError::OwnerChanged);
            }
            let parent_closed = identity(hwnd(self.parent.0)) != Some(self.parent_id)
                || !top_level(hwnd(self.parent.0))?;
            let previous_valid = self.previous_owner.is_null()
                || (self.previous_owner_id.is_some()
                    && identity(self.previous_owner) == self.previous_owner_id
                    && top_level(self.previous_owner)?
                    && valid_owner_chain(self.previous_owner, window)?);
            let previous_owner = if previous_valid {
                self.previous_owner
            } else {
                std::ptr::null_mut()
            };
            set_owner(window, previous_owner)?;
            self.attached = false;
            if previous_valid {
                Ok(if parent_closed {
                    ConfigurationStatus::ParentClosed
                } else {
                    ConfigurationStatus::Released
                })
            } else {
                Err(HostError::PreviousOwnerClosed)
            }
        }
    }

    impl Drop for ConfigurationOwner {
        fn drop(&mut self) {
            let _ = self.restore();
            if let Some(error) = self.cleanup_error {
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "screensaver configuration ownership cleanup: {error}"
                );
            }
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
    use super::{ConfigurationStatus, HostError, ParentWindow, PreviewStatus};

    pub(super) fn validate_parent(_parent: ParentWindow) -> Result<(), HostError> {
        Err(HostError::UnsupportedPlatform)
    }

    pub struct OwnWindow;
    impl OwnWindow {
        pub fn capture_active() -> Result<Self, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
        pub fn has_focus(&self) -> bool {
            false
        }
        pub fn attach_configuration(
            &self,
            _parent: ParentWindow,
        ) -> Result<ConfigurationOwner, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
        pub fn attach_preview(
            &self,
            _parent: ParentWindow,
        ) -> Result<PreviewAttachment, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
    }
    #[must_use = "retain the guard until configuration closes"]
    pub struct ConfigurationOwner;
    impl ConfigurationOwner {
        pub fn poll(&self) -> Result<ConfigurationStatus, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
        pub fn restore(&mut self) -> Result<ConfigurationStatus, HostError> {
            Err(HostError::UnsupportedPlatform)
        }
        pub fn cleanup_error(&self) -> Option<HostError> {
            None
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
