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
    create_file: extern "C" fn (&mut A, *const u8, usize, i32) -> AccessorResult,
    delete_file: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    create_directory: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    delete_directory: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    delete_directory_recursively: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    clean_directory_recursively: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    rename_file: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    rename_directory: extern "C" fn (&mut A, *const u8) -> AccessorResult,
    get_entry_type: extern "C" fn (&mut A, &mut FsEntryType, *const u8) -> AccessorResult,
    get_free_space_size: extern "C" fn (&mut A, &mut usize, *const u8) -> AccessorResult,
    get_total_space_size: extern "C" fn (&mut A, &mut usize, *const u8) -> AccessorResult,
    open_file: extern "C" fn (&mut A, &mut &mut A::FAccessor, *const u8, nn::fs::OpenMode) -> AccessorResult, // *mut *mut is actually std::unique_ptr
    open_directory: extern "C" fn (&mut A, &mut &mut A::DAccessor, *const u8, nn::fs::OpenDirectoryMode) -> AccessorResult,
    commit: extern "C" fn (&mut A) -> AccessorResult,
    commit_provisionally: extern "C" fn (&mut A, u64) -> AccessorResult,
    rollback: extern "C" fn (&mut A) -> AccessorResult,
    flush: extern "C" fn (&mut A) -> AccessorResult,
    get_file_time_stamp_raw: extern "C" fn (&mut A, *mut u64, *const u8) -> AccessorResult, // takes *mut nn::fs::FileTimeStampRaw
    query_entry: extern "C" fn (&mut A,) -> AccessorResult // more args but idgaf 
}

impl<A: FsAccessor> FsAccessorVtable<A> {
    fn new() -> Self {
        Self {
            destructor: A::destructor,
            deleter: A::deleter,
            create_file: A::create_file,
            delete_file: A::delete_file,
            create_directory: A::create_directory,
            delete_directory: A::delete_directory,
            delete_directory_recursively: A::delete_directory_recursively,
            clean_directory_recursively: A::clean_directory_recursively,
            rename_file: A::rename_file,
            rename_directory: A::rename_directory,
            get_entry_type: A::get_entry_type,
            get_free_space_size: A::get_free_space_size,
            get_total_space_size: A::get_total_space_size,
            open_file: A::open_file,
            open_directory: A::open_directory,
            commit: A::commit,
            commit_provisionally: A::commit_provisionally,
            rollback: A::rollback,
            flush: A::flush,
            get_file_time_stamp_raw: A::get_file_time_stamp_raw,
            query_entry: A::query_entry,
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
    extern "C" fn create_file(&mut self, path: *const u8, size: usize, mode: i32) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn delete_file(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn create_directory(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn delete_directory(&mut self, path: *const u8) -> AccessorResult;
    extern "C" fn delete_directory_recursively(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn clean_directory_recursively(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn rename_file(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn rename_directory(&mut self, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_entry_type(&mut self, entry_type: &mut FsEntryType, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_free_space_size(&mut self, out_size: &mut usize, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_total_space_size(&mut self, out_size: &mut usize, path: *const u8) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn open_file(&mut self, file_accessor: &mut &mut Self::FAccessor, path: *const u8, mode: nn::fs::OpenMode) -> AccessorResult { // unique_accessor is actually std::unique_ptr
        let accessor = FileAccessor::<Self::FAccessor>::new();
        *file_accessor = unsafe { &mut *(accessor as *mut Self::FAccessor) };

        AccessorResult::Ok
    }
    extern "C" fn open_directory(&mut self, directory_accessor: &mut &mut Self::DAccessor, path: *const u8, mode: nn::fs::OpenDirectoryMode) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn commit(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn commit_provisionally(&mut self, arg: u64) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn rollback(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn flush(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_file_time_stamp_raw(&mut self, timestamp_out: *mut u64, path: *const u8) -> AccessorResult { // takes *mut nn::fs::FileTimeStampRaw
        AccessorResult::Unimplemented
    }
    extern "C" fn query_entry(&mut self) -> AccessorResult { // more args but idgaf 
        AccessorResult::Unimplemented
    }
}