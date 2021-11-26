use crate::{ fs, AccessorResult };

use skyline::nn;

#[repr(C)]
pub struct FileAccessor<F: FsFileAccessor> {
    vtable: *const FileAccessorVtable<F>,
}

impl<F: FsFileAccessor> FileAccessor<F> {
    pub fn new() -> *mut Self {
        let out = fs::detail::alloc::<Self>();
        unsafe {
            (*out).vtable = std::boxed::Box::leak(std::boxed::Box::new(FileAccessorVtable::new())) as *const FileAccessorVtable<F>
        }
        out
    }
}

#[repr(C)]
struct FileAccessorVtable<A: FsFileAccessor> {
    // also type info at VTable - 0x8
    destructor: extern "C" fn(&mut A),
    deleter: extern "C" fn(&mut A),
    // this, out_size, offset, buffer, buffer_size, read_options
    do_read: extern "C" fn(&mut A, &mut usize, isize, *mut u8, usize, u32) -> AccessorResult,
    do_write: extern "C" fn(&mut A, isize, *const u8, usize, &nn::fs::WriteOption) -> AccessorResult,
    do_flush: extern "C" fn(&mut A) -> AccessorResult,
    do_set_size: extern "C" fn(&mut A, usize) -> AccessorResult,
    do_get_size: extern "C" fn(&mut A, &mut usize) -> AccessorResult,
    // more here but no clue what they are
    do_operate_range: extern "C" fn(&mut A, ) -> AccessorResult
}

impl<F: FsFileAccessor> FileAccessorVtable<F> {
    fn new() -> Self {
        Self {
            destructor: F::destructor,
            deleter: F::deleter,
            do_read: F::do_read,
            do_write: F::do_write,
            do_flush: F::do_flush,
            do_set_size: F::do_set_size,
            do_get_size: F::do_get_size,
            do_operate_range: F::do_operate_range,
        }
    }
}

pub trait FsFileAccessor {
    // also type info at VTable - 0x8
    extern "C" fn destructor(&mut self);
    extern "C" fn deleter(&mut self) where Self: Sized {
        self.destructor();
        fs::detail::free(self as *mut Self); 
    }
    // this, out_size, offset, buffer, buffer_size, read_options
    extern "C" fn do_read(&mut self, out_size: &mut usize, offset: isize, buffer: *mut u8, buffer_len: usize, read_options: u32) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_write(&mut self, offset: isize, buffer: *const u8, buffer_len: usize, write_options: &nn::fs::WriteOption) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_flush(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_set_size(&mut self, size: usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn do_get_size(&mut self, out_size: &mut usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    // more here but no clue what they are
    extern "C" fn do_operate_range(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
}