// Copyright (C) 2026 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Regression test for https://github.com/OSSystems/compress-tools-rs/issues/142
//!
//! When `uncompress_archive` cannot write file contents (disk full) the error
//! must propagate as a `Result::Err`, not a silent `Ok` over truncated files.
//!
//! The only reliable way to trigger `ENOSPC` without root is to mount a tiny
//! `tmpfs` inside an unprivileged user + mount namespace. The test forks a
//! child, unshares into its own namespaces, mounts a 4 KiB `tmpfs`, and runs
//! the extraction against an archive whose contents cannot fit. If the host
//! kernel disallows unprivileged user namespaces (e.g. some hardened distros
//! or CI sandboxes), the test is skipped rather than failed.

#![cfg(target_os = "linux")]

use compress_tools::{uncompress_archive, Ownership};
use std::{
    ffi::CString,
    fs::{self, File},
    io::Write,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process,
};

const SKIPPED: i32 = 77;
const BUG_REPRODUCED: i32 = 1;
const OK: i32 = 0;

#[test]
fn uncompress_archive_errors_when_target_is_full() {
    let scratch = tempfile::tempdir().expect("tempdir");
    let archive = build_oversize_archive(scratch.path());
    let target = scratch.path().join("target");
    fs::create_dir(&target).unwrap();

    unsafe {
        match libc::fork() {
            -1 => panic!("fork failed: {}", std::io::Error::last_os_error()),
            0 => run_child(&archive, &target),
            pid => {
                let mut status: libc::c_int = 0;
                let waited = libc::waitpid(pid, &mut status, 0);
                assert_eq!(
                    waited, pid,
                    "waitpid failed: {}",
                    std::io::Error::last_os_error()
                );
                assert!(
                    libc::WIFEXITED(status),
                    "child did not exit normally (raw status = {status})"
                );
                match libc::WEXITSTATUS(status) {
                    OK => {}
                    SKIPPED => {
                        eprintln!(
                            "skipping: unprivileged user namespaces or tmpfs \
                             mount not available in this environment"
                        );
                    }
                    BUG_REPRODUCED => panic!(
                        "uncompress_archive returned Ok despite the target \
                         partition being full — see issue #142"
                    ),
                    other => panic!("child exited with unexpected code {other}"),
                }
            }
        }
    }
}

unsafe fn run_child(archive: &Path, target: &Path) -> ! {
    let uid = libc::getuid();
    let gid = libc::getgid();

    if libc::unshare(libc::CLONE_NEWUSER | libc::CLONE_NEWNS) != 0 {
        eprintln!(
            "child: unshare failed: {}",
            std::io::Error::last_os_error()
        );
        process::exit(SKIPPED);
    }

    // Map our real uid/gid to 0 inside the new user namespace so that the
    // subsequent mount() syscall has CAP_SYS_ADMIN over the new mount
    // namespace.
    if !write_proc("/proc/self/uid_map", &format!("0 {uid} 1"))
        || !write_proc("/proc/self/setgroups", "deny")
        || !write_proc("/proc/self/gid_map", &format!("0 {gid} 1"))
    {
        process::exit(SKIPPED);
    }

    // Make the entire namespace's mount propagation private so we don't leak
    // mounts to the host (belt-and-braces; unshare(CLONE_NEWNS) already
    // detaches).
    let slash = CString::new("/").unwrap();
    let empty = CString::new("").unwrap();
    libc::mount(
        empty.as_ptr(),
        slash.as_ptr(),
        std::ptr::null(),
        libc::MS_REC | libc::MS_PRIVATE,
        std::ptr::null(),
    );

    let source = CString::new("tmpfs").unwrap();
    let fstype = CString::new("tmpfs").unwrap();
    let options = CString::new("size=4K").unwrap();
    let target_c = CString::new(target.as_os_str().as_bytes()).unwrap();
    if libc::mount(
        source.as_ptr(),
        target_c.as_ptr(),
        fstype.as_ptr(),
        0,
        options.as_ptr() as *const _,
    ) != 0
    {
        eprintln!(
            "child: mount tmpfs failed: {}",
            std::io::Error::last_os_error()
        );
        process::exit(SKIPPED);
    }

    let mut src = match File::open(archive) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("child: open archive failed: {e}");
            process::exit(SKIPPED);
        }
    };

    let result = uncompress_archive(&mut src, target, Ownership::Ignore);
    eprintln!("child: uncompress_archive result = {result:?}");
    process::exit(if result.is_err() { OK } else { BUG_REPRODUCED });
}

unsafe fn write_proc(path: &str, content: &str) -> bool {
    match File::options().write(true).open(path) {
        Ok(mut f) => match f.write_all(content.as_bytes()) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("child: write to {path} failed: {e}");
                false
            }
        },
        Err(e) => {
            eprintln!("child: open {path} failed: {e}");
            false
        }
    }
}

/// Build a tar archive whose uncompressed contents are several times the size
/// of the 4 KiB tmpfs we will mount in the child.
fn build_oversize_archive(dir: &Path) -> PathBuf {
    let src = dir.join("payload");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("a.bin"), vec![0xaa_u8; 8192]).unwrap();
    fs::write(src.join("b.bin"), vec![0xbb_u8; 8192]).unwrap();

    let archive = dir.join("oversize.tar");
    let status = process::Command::new("tar")
        .arg("-cf")
        .arg(&archive)
        .arg("-C")
        .arg(&src)
        .arg(".")
        .status()
        .expect("tar command must be available on Linux test hosts");
    assert!(status.success(), "tar invocation failed");
    archive
}
