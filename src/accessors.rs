use crate::{ fs, AccessorResult, FsEntryType };

mod file;
mod directory;

pub use file::{ FileAccessor, FsFileAccessor };
pub use directory::{ DirectoryAccessor, FsDirectoryAccessor };

use skyline::nn;

#[repr(C)]
pub struct FileSystemAccessor<A: FsAccessor> {
    vtable: *const FsAccessorVtable<A>,
}

impl<A: FsAccessor> FileSystemAccessor<A> {
    pub fn new() -> *mut Self {
        let out = fs::detail::alloc::<Self>();
        unsafe {
            (*out).vtable = std::boxed::Box::leak(std::boxed::Box::new(FsAccessorVtable::new())) as *const FsAccessorVtable<A>
        }
        out
    }
}

#[repr(C)]
struct FsAccessorVtable<A: FsAccessor> {
    destructor: extern "C" fn (&mut A),
    deleter: extern "C" fn (&mut A),
    do_create_file: extern "C" fn (&mut A, *const u8, usize, i32) -> AccessorResult,
    do_delete_file: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_create_directory: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_delete_directory: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_delete_directory_recursively: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_clean_directory_recursively: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_rename_file: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_rename_directory: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    do_get_entry_type: extern "C" fn (&mut A, &mut FsEntryType, *const u8) -> AccessorResult,
    do_get_free_space_size: extern "C" fn (&mut A, &mut usize, *const u8) -> AccessorResult,
    do_get_total_space_size: extern "C" fn (&mut A, &mut usize, *const u8) -> AccessorResult,
    do_open_file: extern "C" fn (&mut A, &mut &mut A::FAccessor, *const u8, nn::fs::OpenMode) -> AccessorResult, // *mut *mut is actually std::unique_ptr
    do_open_directory: extern "C" fn (&mut A, &mut &mut A::DAccessor, *const u8, nn::fs::OpenDirectoryMode) -> AccessorResult,
    do_commit: extern "C" fn (&mut A) -> AccessorResult,
    do_commit_provisionally: extern "C" fn (&mut A, u64) -> AccessorResult,
    do_rollback: extern "C" fn (&mut A) -> AccessorResult,
    do_flush: extern "C" fn (&mut A) -> AccessorResult,
    do_get_file_time_stamp_raw: extern "C" fn (&mut A, *mut u64, *const u8) -> AccessorResult, // takes *mut nn::fs::FileTimeStampRaw
    do_query_entry: extern "C" fn (&mut A,) -> AccessorResult // more args but idgaf 
}

impl<A: FsAccessor> FsAccessorVtable<A> {
    fn new() -> Self {
        Self {
            destructor: A::destructor,
            deleter: A::deleter,
            do_create_file: A::do_create_file,
            do_delete_file: A::do_delete_file,
            do_create_directory: A::do_create_directory,
            do_delete_directory: A::do_delete_directory,
            do_delete_directory_recursively: A::do_delete_directory_recursively,
            do_clean_directory_recursively: A::do_clean_directory_recursively,
            do_rename_file: A::do_rename_file,
            do_rename_directory: A::do_rename_directory,
            do_get_entry_type: A::do_get_entry_type,
            do_get_free_space_size: A::do_get_free_space_size,
            do_get_total_space_size: A::do_get_total_space_size,
            do_open_file: A::do_open_file,
            do_open_directory: A::do_open_directory,
            do_commit: A::do_commit,
            do_commit_provisionally: A::do_commit_provisionally,
            do_rollback: A::do_rollback,
            do_flush: A::do_flush,
            do_get_file_time_stamp_raw: A::do_get_file_time_stamp_raw,
            do_query_entry: A::do_query_entry,
        }
    }
}

pub trait FsAccessor {
    type FAccessor: FsFileAccessor;
    type DAccessor: FsDirectoryAccessor;

    extern "C" fn destructor(&mut self);
    extern "C" fn deleter(&mut self) where Self: Sized {
        self.destructor();
        fs::detail::free(self as *mut Self);   
    }
    extern "C" fn do_create_file(&mut self, path: *const u8, size: usize, mode: i32) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_delete_file(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_create_directory(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_delete_directory(&mut self, path: *const u8) -> AccessorResult;
    extern "C" fn do_delete_directory_recursively(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_clean_directory_recursively(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_rename_file(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_rename_directory(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_get_entry_type(&mut self, entry_type: &mut FsEntryType, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_get_free_space_size(&mut self, out_size: &mut usize, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_get_total_space_size(&mut self, out_size: &mut usize, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_open_file(&mut self, file_accessor: &mut &mut Self::FAccessor, path: *const u8, mode: nn::fs::OpenMode) -> AccessorResult { // unique_accessor is actually std::unique_ptr
        let accessor = FileAccessor::<Self::FAccessor>::new();
        *file_accessor = unsafe { &mut *(accessor as *mut Self::FAccessor) };

        AccessorResult::Ok
    }
    extern "C" fn do_open_directory(&mut self, directory_accessor: &mut &mut Self::DAccessor, path: *const u8, mode: nn::fs::OpenDirectoryMode) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_commit(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_commit_provisionally(&mut self, arg: u64) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_rollback(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_flush(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_get_file_time_stamp_raw(&mut self, timestamp_out: *mut u64, path: *const u8) -> AccessorResult { // takes *mut nn::fs::FileTimeStampRaw
        AccessorResult::Unimplemented
    }
    extern "C" fn do_query_entry(&mut self) -> AccessorResult { // more args but idgaf 
        AccessorResult::Unimplemented
    }
}