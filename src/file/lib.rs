// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![crate_name = "linux_file"]
#![crate_type = "lib"]
#![allow(trivial_numeric_casts)]

extern crate linux_core as core;
extern crate linux_dev as dev;
extern crate linux_fs as fs;
extern crate linux_clock as clock;

use std::{mem};

use core::result::{Result};
use core::errno::{self, Errno};
use core::cty::{self, c_int, off_t, c_uint, AT_FDCWD, AT_EMPTY_PATH, AT_SYMLINK_NOFOLLOW,
                F_OK};
use core::ext::{AsLinuxPath, UIntRange};
use core::syscall::{openat, read, write, close, pread, lseek, pwrite, readv, writev,
                    preadv, pwritev, ftruncate, fsync, fdatasync, syncfs, fadvise,
                    fstatfs, fcntl_dupfd_cloexec, fcntl_getfl, fcntl_setfl, fcntl_getfd,
                    fcntl_setfd, fstatat, faccessat};
use core::util::{retry, empty_cstr};

use fs::info::{FileSystemInfo, from_statfs};

use flags::{Flags, AccessMode, access_mode_to_int};
use info::{Info, info_from_stat};

pub mod flags;
pub mod info;

macro_rules! rv {
    ($x:expr) => { if $x < 0 { Err(Errno(-$x as c_int)) } else { Ok(()) } };
    ($x:expr, -> $t:ty) => { if $x < 0 { Err(Errno(-$x as c_int)) } else { Ok($x as $t) } };
}

/// Returns information about the file specified by `path`.
///
/// If `path` is a symlink, then this is equivalent to returning information about the
/// destination of the symlink. Relative paths will be interpreted relative to the current
/// working directory.
pub fn _info<P: AsLinuxPath>(path: P) -> Result<Info> {
    File::current_dir().rel_info(path)
}

/// Returns information about the file specified by `path`.
///
/// This returns information about the file at `path`, even if `path` is a symlink.
/// Relative paths will be interpreted relative to the current working directory.
pub fn info_no_follow<P: AsLinuxPath>(path: P) -> Result<Info> {
    File::current_dir().rel_info_no_follow(path)
}

/// Returns whether the specified path points to an existing file.
///
/// If `path` is relative then the path will be interpreted relative to the current
/// working directory.
pub fn exists<P: AsLinuxPath>(path: P) -> Result<bool> {
    File::current_dir().rel_exists(path)
}

pub fn can_access<P: AsLinuxPath>(path: P, mode: AccessMode) -> Result<bool> {
    File::current_dir().rel_can_access(path, mode)
}


/// An opened file in a file system.
#[derive(Debug, Eq, PartialEq)]
pub struct File {
    fd: c_int,
    /// File has ownership of the file descriptor.
    owned: bool,
}

impl File {
    /// Creates a file on which every operation fails.
    pub fn invalid() -> File {
        File { fd: -1, owned: false }
    }

    /// Creates a file that points to the current directory.
    pub fn current_dir() -> File {
        File { fd: AT_FDCWD, owned: false }
    }

    /// Returns the file descriptor of this file.
    pub fn file_desc(&self) -> c_int {
        self.fd
    }

    /// Open the file at path `path` with the specified flags.
    ///
    /// If `path` is relative, the `self` must be a directory and the `path` will be
    /// interpreted relative to `self`.
    pub fn rel_open<P: AsLinuxPath>(&self, path: P, flags: Flags) -> Result<File> {
        let path = path.to_cstring().unwrap();
        let fd = match retry(|| openat(self.fd, &path, *flags | cty::O_LARGEFILE,
                                       *flags.mode())) {
            Ok(fd) => fd,
            // Due to a bug in the kernel, open returns WrongDeviceType instead of
            // NoSuchDevice.
            Err(errno::WrongDeviceType) => return Err(errno::NoSuchDevice),
            Err(e) => return Err(e),
        };
        Ok(File {
            fd: fd,
            owned: true,
        })
    }

    /// Opens the file at path `path` in read mode.
    ///
    /// If `path` is relative, the `self` must be a directory and the `path` will be
    /// interpreted relative to `self`.
    ///
    /// This is equivalent to `file.open` with the default flags.
    pub fn rel_open_read<P: AsLinuxPath>(&self, path: P) -> Result<File> {
        self.rel_open(path, Flags::new())
    }

    /// Opens the file at path `path` in read mode.
    ///
    /// This is equivalent to `File::open` with the default flags.
    pub fn open_read<P: AsLinuxPath>(path: P) -> Result<File> {
        File::current_dir().rel_open_read(path)
    }

    /// Open the file at path `path` with the specified flags.
    pub fn open<P: AsLinuxPath>(path: P, flags: Flags) -> Result<File> {
        File::current_dir().rel_open(path, flags)
    }

    /// Reads bytes from the current read position into the buffer.
    ///
    /// ### Return value
    ///
    /// Returns the number of bytes read or an error. Zero indicates end of file.
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        retry(|| read(self.fd, buf)).map(|r| r as usize)
    }

    /// Writes bytes to the current to position from the buffer.
    ///
    /// ### Return value
    ///
    /// Returns the number of bytes written or an error.
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        retry(|| write(self.fd, buf)).map(|r| r as usize)
    }

    /// Closes the file descriptor.
    pub fn close(&mut self) -> Result<()> {
        if self.owned {
            let ret = close(self.fd);
            self.fd = -1;
            rv!(ret)
        } else {
            Ok(())
        }
    }

    /// Returns information about the file.
    pub fn info(&self) -> Result<Info> {
        let mut stat = unsafe { mem::zeroed() };
        try!(rv!(fstatat(self.fd, empty_cstr(), &mut stat, AT_EMPTY_PATH)));
        Ok(info_from_stat(stat))
    }

    /// Returns information about the file specified by `path`.
    ///
    /// If `path` is a symlink, then this is equivalent to returning information about the
    /// destination of the symlink. If `path` is relative, then `self` must be a directory
    /// and the path will be interpreted relative to `self`.
    pub fn rel_info<P: AsLinuxPath>(&self, path: P) -> Result<Info> {
        let path = path.to_cstring().unwrap();
        let mut stat = unsafe  { mem::zeroed() };
        try!(rv!(fstatat(self.fd, &path, &mut stat, 0)));
        Ok(info_from_stat(stat))
    }

    /// Returns information about the file specified by `path`.
    ///
    /// This returns information about the file at `path`, even if `path` is a symlink.
    /// If `path` is relative, then `self` must be a directory and the path will be
    /// interpreted relative to `self`.
    pub fn rel_info_no_follow<P: AsLinuxPath>(&self, path: P) -> Result<Info> {
        let path = path.to_cstring().unwrap();
        let mut stat = unsafe  { mem::zeroed() };
        try!(rv!(fstatat(self.fd, &path, &mut stat, AT_SYMLINK_NOFOLLOW)));
        Ok(info_from_stat(stat))
    }

    /// Performs requested seek operation.
    ///
    /// ### Return value
    ///
    /// Returns the new position in the file or an error.
    pub fn seek(&self, pos: Seek) -> Result<i64> {
        let ret = lseek(self.fd, pos.offset(), pos.whence());
        rv!(ret, -> i64)
    }

    /// Creates a new file referring to the same file description.
    ///
    /// The `close on exec` flag will be set on the new file.
    ///
    /// ### Return value
    ///
    /// Returns the new file or an error.
    pub fn duplicate(&self) -> Result<File> {
        let new_fd = fcntl_dupfd_cloexec(self.fd, 0);
        if new_fd < 0 {
            Err(Errno(-new_fd as c_int))
        } else {
            Ok(File { fd: new_fd, owned: true })
        }
    }

    /// Retrieves the file description flags.
    ///
    /// The returned flags will contain the access mode flags and the file status flags.
    pub fn get_status_flags(&self) -> Result<Flags> {
        let ret = fcntl_getfl(self.fd);
        if ret < 0 {
            Err(Errno(-ret as c_int))
        } else {
            Ok(Flags::from_int(ret))
        }
    }

    /// Sets the file description flags.
    ///
    /// Only the file status flags can be modified.
    pub fn set_status_flags(&self, flags: Flags) -> Result<()> {
        let ret = fcntl_setfl(self.fd, *flags);
        rv!(ret)
    }

    /// Returns whether the file has the `close on exec` flag set.
    pub fn is_close_on_exec(&self) -> Result<bool> {
        let ret = fcntl_getfd(self.fd);
        if ret < 0 {
            Err(Errno(-ret as c_int))
        } else {
            Ok(ret & cty::O_CLOEXEC != 0)
        }
    }

    /// Modifies the `close on exec` flag of the file.
    pub fn set_close_on_exec(&self, val: bool) -> Result<()> {
        let mut ret = fcntl_getfd(self.fd);
        if ret >= 0 {
            ret = (ret & !cty::O_CLOEXEC) | (cty::O_CLOEXEC * val as c_int);
            ret = fcntl_setfd(self.fd, ret);
        }
        rv!(ret)
    }

    /// Reads bytes from the offset into the buffer.
    ///
    /// The return value and errors are the same as for `read` and `seek`.
    pub fn read_at(&self, buf: &mut [u8], off: i64) -> Result<usize> {
        retry(|| pread(self.fd, buf, off as off_t)).map(|r| r as usize)
    }

    /// Writes bytes to the offset from the buffer.
    ///
    /// The return value and errors are the same as for `write` and `seek`.
    pub fn write_at(&self, buf: &[u8], off: i64) -> Result<usize> {
        retry(|| pwrite(self.fd, buf, off as off_t)).map(|r| r as usize)
    }

    /// Reads bytes from the current read position into the buffers.
    ///
    /// ### Return value
    ///
    /// Returns the total number of bytes read.
    pub fn scatter_read(&self, bufs: &mut [&mut [u8]]) -> Result<usize> {
        assert!(bufs.len() < (!0 as c_uint / 2) as usize);
        retry(|| readv(self.fd, bufs)).map(|r| r as usize)
    }

    /// Writes bytes to the current write position from the buffers.
    ///
    /// ### Return value
    ///
    /// Returns the total number of bytes written.
    pub fn gather_write(&self, bufs: &[&[u8]]) -> Result<usize> {
        assert!(bufs.len() < (!0 as c_uint / 2) as usize);
        retry(|| writev(self.fd, bufs)).map(|r| r as usize)
    }

    /// Reads bytes from the offset into the buffers.
    ///
    /// ### Return value
    ///
    /// Returns the total number of bytes read.
    pub fn scatter_read_at(&self, bufs: &mut [&mut [u8]], off: i64) -> Result<usize> {
        assert!(bufs.len() < (!0 as c_uint / 2) as usize);
        retry(|| preadv(self.fd, bufs, off as off_t)).map(|r| r as usize)
    }

    /// Writes bytes to the offset from the buffers.
    ///
    /// ### Return value
    ///
    /// Returns the total number of bytes written.
    pub fn gather_write_at(&self, bufs: &[&[u8]], off: i64) -> Result<usize> {
        assert!(bufs.len() < (!0 as c_uint / 2) as usize);
        retry(|| pwritev(self.fd, bufs, off as off_t)).map(|r| r as usize)
    }

    /// Changes the length of the file to the specified length.
    ///
    /// If the requested length is larger than the current length, a hole is created.
    pub fn set_len(&self, len: i64) -> Result<()> {
        retry(|| ftruncate(self.fd, len as off_t)).map(|_| ())
    }

    /// Flushes all data and metadata to the disk.
    pub fn sync(&self) -> Result<()> {
        rv!(fsync(self.fd))
    }

    /// Flushes enough data to the disk that the content of the file can be read again.
    pub fn data_sync(&self) -> Result<()> {
        rv!(fdatasync(self.fd))
    }

    /// Writes all data and metadata of the filesystem containing this file to the disk.
    pub fn sync_filesystem(&self) -> Result<()> {
        rv!(syncfs(self.fd))
    }

    /// Advise the kernel that the specified range will have a certain usage pattern.
    pub fn advise<R: UIntRange<u64>>(&self, range: R, advice: Advice) -> Result<()> {
        let range = range.to_range();
        let len = match range.end {
            -1 => 0,
            _ => range.end - range.start,
        };
        let ret = fadvise(self.fd, range.start as off_t, len as off_t, advice.to_c_int());
        rv!(ret)
    }

    /// Returns information about the file system of this file.
    pub fn fs_info(&self) -> Result<FileSystemInfo> {
        let mut buf = unsafe { mem::zeroed() };
        retry(|| fstatfs(self.fd, &mut buf)).map(|_| from_statfs(buf))
    }

    /// Returns whether the specified path points to an existing file.
    ///
    /// If `path` is relative then `self` must be a directory and the path will be
    /// interpreted relative to `self`.
    pub fn rel_exists<P: AsLinuxPath>(&self, path: P) -> Result<bool> {
        let path = path.to_cstring().unwrap();
        let res = faccessat(self.fd, &path, F_OK);
        if res >= 0 {
            Ok(true)
        } else {
            let err = Errno(-res);
            match err {
                errno::DoesNotExist => Ok(false),
                _ => Err(err),
            }
        }
    }

    pub fn rel_can_access<P: AsLinuxPath>(&self, path: P, mode: AccessMode) -> Result<bool> {
        let path = path.to_cstring().unwrap();
        let res = faccessat(self.fd, &path, access_mode_to_int(mode));
        if res >= 0 {
            Ok(true)
        } else {
            let err = Errno(-res);
            match err {
                errno::AccessDenied => Ok(false),
                _ => Err(err),
            }
        }
    }
}

impl ::std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        Ok(try!(File::read(self, buf)))
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if self.owned {
            close(self.fd);
        }
    }
}

/// A seek operation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Seek {
    /// Seek from the start of the file.
    Start(i64),
    /// Seek from the current position in the file.
    Cur(i64),
    /// Seek from the end of the file.
    End(i64),
    /// Seek to the first non-hole byte at or after the specified offset.
    Data(i64),
    /// Seek to the first hole at or after the specified offset.
    Hole(i64),
}

impl Seek {
    fn whence(self) -> c_int {
        match self {
            Seek::Start(..) => 0,
            Seek::Cur(..)   => 1,
            Seek::End(..)   => 2,
            Seek::Data(..)  => 3,
            Seek::Hole(..)  => 4,
        }
    }

    fn offset(self) -> off_t {
        match self {
            Seek::Start(v) => v as off_t,
            Seek::Cur(v)   => v as off_t,
            Seek::End(v)   => v as off_t,
            Seek::Data(v)  => v as off_t,
            Seek::Hole(v)  => v as off_t,
        }
    }
}

/// Advice used to optimize file access.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Advice {
    /// Default.
    Normal,
    /// Optimize for random access.
    Random,
    /// Optimize for sequential access.
    Sequential,
    /// The range will be accessed soon.
    Need,
    /// The range will not be accessed.
    DontNeed,
    /// The range will be accessed only once.
    NoReuse,
}

impl Advice {
    fn to_c_int(self) -> c_int {
        match self {
            Advice::Normal => 0,
            Advice::Random => 1,
            Advice::Sequential => 2,
            Advice::Need => 3,
            Advice::DontNeed => 4,
            Advice::NoReuse => 5,
        }
    }
}
