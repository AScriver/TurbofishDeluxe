//! Small original Rust binding to the libopenmpt 0.8.9 C API. No game state or audio device.
//! ABI reference: official `inc/libopenmpt/libopenmpt.h` from the VS2022 x64 package.
#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    ffi::{c_char, c_int, c_void, CStr},
    fmt,
    marker::PhantomData,
    ptr::{self, NonNull},
    rc::Rc,
};

#[repr(C)]
struct NativeModule {
    _private: [u8; 0],
}

type LogFunction = unsafe extern "C" fn(*const c_char, *mut c_void);
type ErrorFunction = unsafe extern "C" fn(c_int, *mut c_void) -> c_int;

#[link(name = "libopenmpt")]
unsafe extern "C" {
    fn openmpt_module_create_from_memory2(
        filedata: *const c_void,
        filesize: usize,
        logfunc: Option<LogFunction>,
        loguser: *mut c_void,
        errfunc: Option<ErrorFunction>,
        erruser: *mut c_void,
        error: *mut c_int,
        error_message: *mut *const c_char,
        ctls: *const c_void,
    ) -> *mut NativeModule;
    fn openmpt_log_func_silent(message: *const c_char, user: *mut c_void);
    fn openmpt_error_func_store(error: c_int, user: *mut c_void) -> c_int;
    fn openmpt_error_string(error: c_int) -> *const c_char;
    fn openmpt_free_string(message: *const c_char);
    fn openmpt_module_destroy(module: *mut NativeModule);
    fn openmpt_module_error_get_last(module: *mut NativeModule) -> c_int;
    fn openmpt_module_error_get_last_message(module: *mut NativeModule) -> *const c_char;
    fn openmpt_module_error_clear(module: *mut NativeModule);
    fn openmpt_module_get_num_orders(module: *mut NativeModule) -> i32;
    fn openmpt_module_get_current_order(module: *mut NativeModule) -> i32;
    fn openmpt_module_get_current_row(module: *mut NativeModule) -> i32;
    fn openmpt_module_set_position_order_row(
        module: *mut NativeModule,
        order: i32,
        row: i32,
    ) -> f64;
    fn openmpt_module_set_repeat_count(module: *mut NativeModule, repeats: i32) -> c_int;
    fn openmpt_module_read_interleaved_float_stereo(
        module: *mut NativeModule,
        samplerate: i32,
        frames: usize,
        output: *mut f32,
    ) -> usize;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput(&'static str),
    Native {
        operation: &'static str,
        code: i32,
        message: String,
    },
    UnexpectedResult(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::UnexpectedResult(message) => {
                formatter.write_str(message)
            }
            Self::Native {
                operation,
                code,
                message,
            } => write!(formatter, "{operation}: libopenmpt error {code}: {message}"),
        }
    }
}

impl std::error::Error for Error {}

fn take_native_string(pointer: *const c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    // libopenmpt owns and allocates these strings. Copy before freeing with its own allocator.
    let value = unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned();
    unsafe { openmpt_free_string(pointer) };
    Some(value)
}

fn error_text(code: i32, message: *const c_char) -> String {
    take_native_string(message)
        .or_else(|| take_native_string(unsafe { openmpt_error_string(code) }))
        .unwrap_or_else(|| "no error text available".to_owned())
}

/// A decoder confined to one thread. The native module has mutable playback state.
pub struct Module {
    native: NonNull<NativeModule>,
    _not_send_or_sync: PhantomData<Rc<()>>,
}

impl Module {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidInput("module bytes are empty"));
        }
        let mut code = 0;
        let mut message = ptr::null();
        // The official header says input bytes may be discarded after successful construction.
        // Both callbacks are exported by the same DLL and need no Rust-owned user state.
        let native = unsafe {
            openmpt_module_create_from_memory2(
                bytes.as_ptr().cast(),
                bytes.len(),
                Some(openmpt_log_func_silent),
                ptr::null_mut(),
                Some(openmpt_error_func_store),
                ptr::null_mut(),
                &mut code,
                &mut message,
                ptr::null(),
            )
        };
        let message = error_text_if_present(code, message);
        let Some(native) = NonNull::new(native) else {
            return Err(Error::Native {
                operation: "load module",
                code,
                message: message.unwrap_or_else(|| "native constructor returned null".to_owned()),
            });
        };
        let mut module = Self {
            native,
            _not_send_or_sync: PhantomData,
        };
        if code != 0 {
            // An error with a non-null module is still an error; Drop destroys it once.
            return Err(Error::Native {
                operation: "load module",
                code,
                message: message
                    .unwrap_or_else(|| "native constructor reported an error".to_owned()),
            });
        }
        module.check("load module")?;
        Ok(module)
    }

    fn clear_error(&mut self) {
        unsafe { openmpt_module_error_clear(self.native.as_ptr()) };
    }

    fn check(&mut self, operation: &'static str) -> Result<(), Error> {
        let code = unsafe { openmpt_module_error_get_last(self.native.as_ptr()) };
        if code == 0 {
            return Ok(());
        }
        let message = unsafe { openmpt_module_error_get_last_message(self.native.as_ptr()) };
        let text = error_text(code, message);
        self.clear_error();
        Err(Error::Native {
            operation,
            code,
            message: text,
        })
    }

    pub fn order_count(&mut self) -> Result<u32, Error> {
        self.clear_error();
        let count = unsafe { openmpt_module_get_num_orders(self.native.as_ptr()) };
        self.check("read order count")?;
        u32::try_from(count)
            .ok()
            .filter(|count| *count > 0)
            .ok_or(Error::UnexpectedResult(
                "native module reported no valid orders",
            ))
    }

    pub fn current_order(&mut self) -> Result<u32, Error> {
        self.clear_error();
        let order = unsafe { openmpt_module_get_current_order(self.native.as_ptr()) };
        self.check("read current order")?;
        u32::try_from(order).map_err(|_| Error::UnexpectedResult("native order is negative"))
    }

    fn current_row(&mut self) -> Result<u32, Error> {
        self.clear_error();
        let row = unsafe { openmpt_module_get_current_row(self.native.as_ptr()) };
        self.check("read current row")?;
        u32::try_from(row).map_err(|_| Error::UnexpectedResult("native row is negative"))
    }

    pub fn seek(&mut self, order: u32, row: u32) -> Result<f64, Error> {
        let count = self.order_count()?;
        if order >= count || order > i32::MAX as u32 || row > i32::MAX as u32 {
            return Err(Error::InvalidInput("order or row is outside native range"));
        }
        self.clear_error();
        let seconds = unsafe {
            openmpt_module_set_position_order_row(self.native.as_ptr(), order as i32, row as i32)
        };
        self.check("seek")?;
        if !seconds.is_finite() || self.current_order()? != order || self.current_row()? != row {
            return Err(Error::UnexpectedResult(
                "native seek did not reach requested order and row",
            ));
        }
        Ok(seconds)
    }

    /// `-1` loops forever, `0` plays once; positive values add that many repeats.
    pub fn set_repeat_count(&mut self, repeats: i32) -> Result<(), Error> {
        if repeats < -1 {
            return Err(Error::InvalidInput("repeat count is below -1"));
        }
        self.clear_error();
        let accepted = unsafe { openmpt_module_set_repeat_count(self.native.as_ptr(), repeats) };
        self.check("set repeat count")?;
        if accepted != 1 {
            return Err(Error::UnexpectedResult(
                "native repeat control was rejected",
            ));
        }
        Ok(())
    }

    /// Returns frames written. Only `buffer[..frames * 2]` is rendered on success.
    pub fn render_stereo(&mut self, sample_rate: i32, buffer: &mut [f32]) -> Result<usize, Error> {
        if !(8_000..=192_000).contains(&sample_rate) {
            return Err(Error::InvalidInput("sample rate must be 8000..=192000 Hz"));
        }
        if !buffer.len().is_multiple_of(2) {
            return Err(Error::InvalidInput(
                "stereo sample buffer length must be even",
            ));
        }
        let frames = buffer.len() / 2;
        if frames > 16_384 {
            return Err(Error::InvalidInput(
                "render request exceeds 16384 stereo frames",
            ));
        }
        if frames == 0 {
            return Ok(0);
        }
        self.clear_error();
        let rendered = unsafe {
            openmpt_module_read_interleaved_float_stereo(
                self.native.as_ptr(),
                sample_rate,
                frames,
                buffer.as_mut_ptr(),
            )
        };
        self.check("render stereo")?;
        if rendered > frames {
            return Err(Error::UnexpectedResult(
                "native renderer exceeded requested frame count",
            ));
        }
        if !buffer[..rendered * 2]
            .iter()
            .all(|sample| sample.is_finite())
        {
            return Err(Error::UnexpectedResult(
                "native renderer produced non-finite samples",
            ));
        }
        Ok(rendered)
    }
}

fn error_text_if_present(code: i32, message: *const c_char) -> Option<String> {
    if code == 0 && message.is_null() {
        None
    } else {
        Some(error_text(code, message))
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { openmpt_module_destroy(self.native.as_ptr()) };
    }
}
