#![feature(maybe_uninit_extra)]

mod accessors;
pub use accessors::*;

#[repr(u32)]
pub enum FsEntryType {
    Directory = 0,
    File = 1
}

#[repr(u32)]
pub enum AccessorResult {
    Success = 0,
    PathNotFound = 0x202,
    PathAlreadyExists = 0x402,
    AlreadyInUse = 0xe02,
    DirectoryNotEmpty = 0x1002,
    OutOfSpace = 0x3c02,
    Unimplemented = 0x177202,
    Unexpected = 0x271002,
    Unsupported = 0x31b802,
}

pub mod fs {
    pub mod detail {
        use skyline::libc::{c_char, c_void};
        use std::mem::MaybeUninit;

        extern "C" {
            #[link_name = "\u{1}_ZN2nn2fs6detail8AllocateEm"]
            fn allocate(size: usize) -> *mut c_void;

            #[link_name = "\u{1}_ZN2nn2fs6detail10DeallocateEPvm"]
            fn deallocate(ptr: *mut c_void);

            #[link_name = "\u{1}_ZN2nn2fs6detail14CheckMountNameEPKc"]
            fn check_mount_name(name: *const c_char) -> u32;
        }

        pub fn alloc<T: Sized>() -> *mut T {
            unsafe {
                allocate(std::mem::size_of::<T>()) as *mut T
            }
        }

        pub fn free<T>(ptr: *mut T) {
            unsafe {
                deallocate(ptr as *mut c_void)
            }
        }

        pub fn is_mount_available<S: AsRef<str>>(name: S) -> bool {
            unsafe {
                check_mount_name([name.as_ref(), "\0"].concat().as_ptr()) == 0
            }
        }
    }

    pub mod fsa {
        use skyline::libc::c_char;

        extern "C" {
            #[link_name = "\u{1}_ZN2nn2fs3fsa8RegisterEPKcONSt3__110unique_ptrINS1_11IFileSystemENS4_14default_deleteIS6_EEEE"]
            fn register_fsa(mount_name: *const c_char, unique_fs_ptr: *mut *mut u8) -> u32;
        }

        pub fn register<S: AsRef<str>, T>(mount_name: S, fsa: *mut T) -> u32 {
            unsafe {
                register_fsa([mount_name.as_ref(), "\0"].concat().as_ptr(), &mut (fsa as *mut u8))
            }
        }
    }
}

pub fn mount(mount_name: &str, accessor: &mut FsAccessor) -> Result<(), std::io::Error>{
    if fs::detail::is_mount_available(mount_name) {
        if fs::fsa::register(mount_name, accessor) != 0 {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to mount the filesystem accessor"))
        } else {
            Ok(())
        }
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "The mount point provided is unavailable"))
    }
}