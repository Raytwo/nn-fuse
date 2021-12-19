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
    Ok = 0,
    PathDoNotExists = 0x202,
    Unimplemented = 0x177202,
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
            #[link_name = "\u{1}_ZN2nn2fs3fsa8RegisterEPKcPNS1_18IMultiCommitTargetEONSt3__110unique_ptrINS1_11IFileSystemENS6_14default_deleteIS8_EEEEONS7_INS1_25ICommonMountNameGeneratorENS9_ISD_EEEEbb"]
            fn register_fsa(mount_name: *const c_char, commit_target: *mut u8, unique_fs_ptr: *mut *mut u8, unique_generator_ptr: *mut *mut u8, unk1: bool, unk2: bool) -> u32;
        }

        pub fn register<S: AsRef<str>, T>(mount_name: S, fsa: *mut T) -> u32 {
            unsafe {
                register_fsa([mount_name.as_ref(), "\0"].concat().as_ptr(), 0 as _, &mut (fsa as *mut u8), &mut (0 as *mut u8), true, true)
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