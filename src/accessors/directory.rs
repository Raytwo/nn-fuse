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
    add_reference: extern "C" fn(&mut D),
    release: extern "C" fn(&mut D),
    get_proxy_info: extern "C" fn(&mut D),
    get_interface_type_info: extern "C" fn(&mut D),
    read: extern "C" fn(&mut D, &mut isize, *mut nn::fs::DirectoryEntry, usize) -> AccessorResult,
    get_entry_count: extern "C" fn(&mut D, &mut isize) -> AccessorResult
}

impl<D: FsDirectoryAccessor> DirectoryAccessorVtable<D> {
    fn new() -> Self {
        Self {
            add_reference: D::add_reference,
            release: D::release,
            get_proxy_info: D::get_proxy_info,
            get_interface_type_info: D::get_interface_type_info,
            read: D::read,
            get_entry_count: D::get_entry_count,
        }
    }
}

pub trait FsDirectoryAccessor {
    // also type info at VTable - 0x8
    extern "C" fn add_reference(&mut self);
    extern "C" fn release(&mut self);
    extern "C" fn get_proxy_info(&mut self);
    extern "C" fn get_interface_type_info(&mut self);
    extern "C" fn read(&mut self, out_count: &mut isize, buffer: *mut nn::fs::DirectoryEntry, buffer_len: usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_entry_count(&mut self, out_count: &mut isize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
}