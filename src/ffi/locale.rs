// Copyright (C) 2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Change from the C to system locale, allowing libarchive to handle filenames
/// in UTF-8. We restrict to change LC_CTYPE only, since libarchive only needs
/// the charset set. The timing of locale setting for Unix and Windows is
/// different, handle them separately.
///
/// See on libarchive Website for a more complete description of the issue:
///
///   https://github.com/libarchive/libarchive/issues/587
///   https://github.com/libarchive/libarchive/wiki/Filenames
pub(crate) use inner::UTF8LocaleGuard;
pub(crate) use inner::WindowsUTF8LocaleGuard;

#[cfg(unix)]
mod inner {
    pub(crate) struct UTF8LocaleGuard {
        save: libc::locale_t,
        utf8_locale: libc::locale_t,
    }

    pub(crate) struct WindowsUTF8LocaleGuard {}

    impl UTF8LocaleGuard {
        pub(crate) fn new() -> Self {
            #[cfg(any(target_os = "linux", target_os = "illumos"))]
            let locale = b"\0";

            #[cfg(target_os = "macos")]
            let locale = b"UTF-8\0";

            let utf8_locale = unsafe {
                libc::newlocale(
                    libc::LC_CTYPE_MASK,
                    locale.as_ptr() as *const libc::c_char,
                    std::ptr::null_mut(),
                )
            };

            let save = if utf8_locale.is_null() {
                std::ptr::null_mut()
            } else {
                unsafe { libc::uselocale(utf8_locale) }
            };

            Self { save, utf8_locale }
        }
    }

    impl Drop for UTF8LocaleGuard {
        fn drop(&mut self) {
            unsafe {
                if !self.utf8_locale.is_null() {
                    libc::uselocale(self.save);
                    libc::freelocale(self.utf8_locale);
                }
            };
        }
    }

    impl WindowsUTF8LocaleGuard {
        pub(crate) fn new() -> Self {
            Self {}
        }
    }
}

#[cfg(windows)]
mod inner {
    extern "C" {
        fn _configthreadlocale(arg1: std::os::raw::c_int) -> std::os::raw::c_int;
    }
    const _ENABLE_PER_THREAD_LOCALE: std::os::raw::c_int = 1;

    pub(crate) struct UTF8LocaleGuard {}

    pub(crate) struct WindowsUTF8LocaleGuard {
        save: Option<std::ffi::CString>,
        save_thread_config: ::std::os::raw::c_int,
    }

    impl UTF8LocaleGuard {
        pub(crate) fn new() -> Self {
            Self {}
        }
    }

    impl WindowsUTF8LocaleGuard {
        pub(crate) fn new() -> Self {
            let locale = b".UTF-8\0";

            let (save, save_thread_config) = {
                let old_locale = unsafe { libc::setlocale(libc::LC_CTYPE, std::ptr::null()) };
                (
                    if old_locale.is_null() {
                        None
                    } else {
                        Some(unsafe { std::ffi::CStr::from_ptr(old_locale) }.to_owned())
                    },
                    unsafe { _configthreadlocale(0) },
                )
            };

            unsafe {
                _configthreadlocale(_ENABLE_PER_THREAD_LOCALE);
                libc::setlocale(
                    libc::LC_CTYPE,
                    std::ffi::CStr::from_bytes_with_nul_unchecked(locale).as_ptr(),
                )
            };

            Self {
                save,
                save_thread_config,
            }
        }
    }

    impl Drop for WindowsUTF8LocaleGuard {
        fn drop(&mut self) {
            if let Some(locale) = &self.save {
                unsafe { libc::setlocale(libc::LC_CTYPE, locale.as_ptr()) };
            }

            unsafe {
                _configthreadlocale(self.save_thread_config);
            }
        }
    }
}
