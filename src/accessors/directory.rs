use crate::{ fs, AccessorResult };

use skyline::nn;

#[repr(C)]
pub struct DirectoryAccessor<D: FsDirectoryAccessor> {
    vtable: *const DirectoryAccessorVtable<D>,
}

impl<D: FsDirectoryAccessor> DirectoryAccessor<D> {
    pub fn new() -> *mut Self {
        let out = fs::detail::alloc::<Self>();
        unsafe {
            (*out).vtable = std::boxed::Box::leak(std::boxed::Box::new(DirectoryAccessorVtable::new())) as *const DirectoryAccessorVtable<D>
        }
        out
    }
}

#[repr(C)]
struct DirectoryAccessorVtable<D: FsDirectoryAccessor> {
    // also type info at VTable - 0x8
    destructor: extern "C" fn(&mut D),
    deleter: extern "C" fn(&mut D),
    read: extern "C" fn(&mut D, &mut isize, *mut nn::fs::DirectoryEntry, usize) -> AccessorResult,
    get_entry_count: extern "C" fn(&mut D, &mut isize) -> AccessorResult
}

impl<D: FsDirectoryAccessor> DirectoryAccessorVtable<D> {
    fn new() -> Self {
        Self {
            destructor: D::destructor,
            deleter: D::deleter,
            read: D::read,
            get_entry_count: D::get_entry_count,
        }
    }
}

pub trait FsDirectoryAccessor {
    // also type info at VTable - 0x8
    extern "C" fn destructor(&mut self);
    extern "C" fn deleter(&mut self) where Self: Sized {
        self.destructor();
        fs::detail::free(self as *mut Self); 
    }
    extern "C" fn read(&mut self, out_count: &mut isize, buffer: *mut nn::fs::DirectoryEntry, buffer_len: usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_entry_count(&mut self, out_count: &mut isize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
}