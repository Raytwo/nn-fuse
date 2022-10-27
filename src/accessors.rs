use crate::{ fs, AccessorResult, FsEntryType };

mod file;
mod directory;

pub use file::{ FileAccessor, FAccessor };
pub use directory::{DAccessor, DirectoryAccessor, DirectoryEntry, DirectoryEntryType };

use std::ffi::CStr;
use skyline::nn;

#[repr(C)]
pub struct nnfsPath {
    pub path: *const u8,
    unk1: [u64;3],
    pub initialize: bool,
}

#[repr(C)]
struct FsAccessorVtable {
    destructor: extern "C" fn (&mut FsAccessor),
    deleter: extern "C" fn (&mut FsAccessor),
    create_file: extern "C" fn (&mut FsAccessor, *const nnfsPath, usize, i32) -> AccessorResult,
    delete_file: extern "C" fn (&mut FsAccessor, *const nnfsPath) -> AccessorResult,
    create_directory: extern "C" fn (&mut FsAccessor, *const nnfsPath) -> AccessorResult,
    delete_directory: extern "C" fn (&mut FsAccessor, *const nnfsPath) -> AccessorResult,
    delete_directory_recursively: extern "C" fn (&mut FsAccessor, *const nnfsPath) -> AccessorResult,
    clean_directory_recursively: extern "C" fn (&mut FsAccessor, *const nnfsPath) -> AccessorResult,
    rename_file: extern "C" fn (&mut FsAccessor, *const nnfsPath, *const nnfsPath) -> AccessorResult,
    rename_directory: extern "C" fn (&mut FsAccessor, *const nnfsPath, *const nnfsPath) -> AccessorResult,
    get_entry_type: extern "C" fn (&mut FsAccessor, &mut FsEntryType, *const nnfsPath) -> AccessorResult,
    get_free_space_size: extern "C" fn (&mut FsAccessor, &mut usize, *const nnfsPath) -> AccessorResult,
    get_total_space_size: extern "C" fn (&mut FsAccessor, &mut usize, *const nnfsPath) -> AccessorResult,
    open_file: extern "C" fn (&mut FsAccessor, *mut *mut FAccessor, *const nnfsPath, nn::fs::OpenMode) -> AccessorResult, // *mut *mut is actually std::unique_ptr
    open_directory: extern "C" fn (&mut FsAccessor, *mut *mut DAccessor, *const nnfsPath, nn::fs::OpenDirectoryMode) -> AccessorResult,
    commit: extern "C" fn (&mut FsAccessor) -> AccessorResult,
    commit_provisionally: extern "C" fn (&mut FsAccessor, u64) -> AccessorResult,
    rollback: extern "C" fn (&mut FsAccessor) -> AccessorResult,
    flush: extern "C" fn (&mut FsAccessor) -> AccessorResult,
    get_file_time_stamp_raw: extern "C" fn (&mut FsAccessor, *mut u64, *const nnfsPath) -> AccessorResult, // takes *mut nn::fs::FileTimeStampRaw
    query_entry: extern "C" fn (&mut FsAccessor,) -> AccessorResult // more args but idgaf 
}

static FSACCESSOR_VTABLE: FsAccessorVtable  = FsAccessorVtable {
    destructor: FsAccessor::destructor,
    deleter: FsAccessor::deleter,
    create_file: FsAccessor::create_file,
    delete_file: FsAccessor::delete_file,
    create_directory: FsAccessor::create_directory,
    delete_directory: FsAccessor::delete_directory,
    delete_directory_recursively: FsAccessor::delete_directory_recursively,
    clean_directory_recursively: FsAccessor::clean_directory_recursively,
    rename_file: FsAccessor::rename_file,
    rename_directory: FsAccessor::rename_directory,
    get_entry_type: FsAccessor::get_entry_type,
    get_free_space_size: FsAccessor::get_free_space_size,
    get_total_space_size: FsAccessor::get_total_space_size,
    open_file: FsAccessor::open_file,
    open_directory: FsAccessor::open_directory,
    commit: FsAccessor::commit,
    commit_provisionally: FsAccessor::commit_provisionally,
    rollback: FsAccessor::rollback,
    flush: FsAccessor::flush,
    get_file_time_stamp_raw: FsAccessor::get_file_time_stamp_raw,
    query_entry: FsAccessor::query_entry,
};

#[repr(C)]
pub struct FsAccessor {
    vtable: &'static FsAccessorVtable,
    accessor: Box<dyn FileSystemAccessor>,
}

impl FsAccessor {
    pub fn new<A: FileSystemAccessor + 'static>(accessor: A) -> *mut Self {
        let out = fs::detail::alloc::<Self>();

        // SAFETY: Do not change this way of assigning the values in case of refactoring. Dereferencing `out` to assign a new instance of the struct would call the destructor for the Box field, which is uninitialized, and cause a crash.
        unsafe {
            out.write(Self {
                vtable: &FSACCESSOR_VTABLE,
                accessor: Box::new(accessor) as _
            });
        }

        let out: *mut FsAccessor = unsafe { std::mem::transmute(out) };
        out
    }

    extern "C" fn destructor(&mut self) {}

    extern "C" fn deleter(&mut self) {
        self.destructor();
        fs::detail::free(self);   
    }

    extern "C" fn get_entry_type(&mut self, entry_type: &mut FsEntryType, path: *const nnfsPath) -> AccessorResult {
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        match self.accessor.get_entry_type(&filepath.strip_prefix("/").unwrap()) {
            Ok(result) => {
                *entry_type = result;
                AccessorResult::Success
            },
            Err(e) => e,
        }
    }

    extern "C" fn create_file(&mut self, path: *const nnfsPath, size: usize, mode: i32) -> AccessorResult {
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        self.accessor.create_file(&filepath, size)
    }
    
    extern "C" fn open_file(&mut self, file_accessor: *mut *mut FAccessor, path: *const nnfsPath, mode: nn::fs::OpenMode) -> AccessorResult { // unique_accessor is actually std::unique_ptr
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        match self.accessor.open_file(&filepath.strip_prefix("/").unwrap(), mode) {
            Ok(mut accessor) => {
                unsafe { *file_accessor = &mut *accessor };
                AccessorResult::Success
            },
            Err(e) => e,
        }
    }

    extern "C" fn rename_file(&mut self, path: *const nnfsPath, new_path: *const nnfsPath) -> AccessorResult {
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };
        let new_filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*new_path).path as _).to_str().unwrap().into() };

        self.accessor.rename_file(&filepath, &new_filepath)
    }

    extern "C" fn delete_file(&mut self, path: *const nnfsPath) -> AccessorResult {
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        self.accessor.delete_file(&filepath)
    }

    extern "C" fn create_directory(&mut self, path: *const nnfsPath) -> AccessorResult {
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        self.accessor.delete_directory(&filepath)
    }

    extern "C" fn open_directory(&mut self, directory_accessor: *mut *mut DAccessor, path: *const nnfsPath, mode: nn::fs::OpenDirectoryMode) -> AccessorResult {
        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        match self.accessor.open_directory(&filepath.strip_prefix("/").unwrap(), mode) {
            Ok(mut accessor) => {
                unsafe { *directory_accessor = &mut *accessor };
                AccessorResult::Success
            },
            Err(e) => e,
        }
    }

    extern "C" fn rename_directory(&mut self, path: *const nnfsPath, new_path: *const nnfsPath) -> AccessorResult {

        let dir_path: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };
        let new_dirpath: std::path::PathBuf = unsafe { CStr::from_ptr((*new_path).path as _).to_str().unwrap().into() };

        self.accessor.rename_directory(&dir_path, &new_dirpath)
    }

    extern "C" fn delete_directory(&mut self, path: *const nnfsPath) -> AccessorResult {

        let filepath: std::path::PathBuf = unsafe { CStr::from_ptr((*path).path as _).to_str().unwrap().into() };

        self.accessor.delete_directory(&filepath)
    }

    extern "C" fn delete_directory_recursively(&mut self, path: *const nnfsPath) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn clean_directory_recursively(&mut self, path: *const nnfsPath) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn get_free_space_size(&mut self, out_size: &mut usize, path: *const nnfsPath) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn get_total_space_size(&mut self, out_size: &mut usize, path: *const nnfsPath) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn commit(&mut self) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn commit_provisionally(&mut self, arg: u64) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn rollback(&mut self) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn flush(&mut self) -> AccessorResult {
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }

    extern "C" fn get_file_time_stamp_raw(&mut self, timestamp_out: *mut u64, path: *const nnfsPath) -> AccessorResult { // takes *mut nn::fs::FileTimeStampRaw
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }
    
    extern "C" fn query_entry(&mut self) -> AccessorResult { // more args but idgaf 
        panic!("FsAccessor");

        AccessorResult::Unimplemented
    }
}

pub trait FileSystemAccessor {
    fn get_entry_type(&self, path: &std::path::Path) -> Result<FsEntryType, AccessorResult>;
    fn create_file(&self, path: &std::path::Path, size: usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    fn open_file(&self, path: &std::path::Path, mode: nn::fs::OpenMode) -> Result<*mut FAccessor, AccessorResult>;
    fn rename_file(&self, path: &std::path::Path, new_path: &std::path::Path) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    fn delete_file(&self, path: &std::path::Path) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    fn create_directory(&self, path: &std::path::Path) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    fn open_directory(&self, path: &std::path::Path, mode: nn::fs::OpenDirectoryMode) -> Result<*mut DAccessor, AccessorResult>;
    fn rename_directory(&self, path: &std::path::Path, new_path: &std::path::Path) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    fn delete_directory(&self, path: &std::path::Path) -> AccessorResult {
        AccessorResult::Unimplemented
    }
}