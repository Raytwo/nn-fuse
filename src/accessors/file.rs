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
    read: extern "C" fn(&mut A, &mut usize, isize, *mut u8, usize, u32) -> AccessorResult,
    write: extern "C" fn(&mut A, isize, *const u8, usize, &nn::fs::WriteOption) -> AccessorResult,
    flush: extern "C" fn(&mut A) -> AccessorResult,
    set_size: extern "C" fn(&mut A, usize) -> AccessorResult,
    get_size: extern "C" fn(&mut A, &mut usize) -> AccessorResult,
    // more here but no clue what they are
    operate_range: extern "C" fn(&mut A, ) -> AccessorResult
}

impl<F: FsFileAccessor> FileAccessorVtable<F> {
    fn new() -> Self {
        Self {
            destructor: F::destructor,
            deleter: F::deleter,
            read: F::read,
            write: F::write,
            flush: F::flush,
            set_size: F::set_size,
            get_size: F::get_size,
            operate_range: F::operate_range,
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
    extern "C" fn read(&mut self, out_size: &mut usize, offset: isize, buffer: *mut u8, buffer_len: usize, read_options: u32) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn write(&mut self, offset: isize, buffer: *const u8, buffer_len: usize, write_options: &nn::fs::WriteOption) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn flush(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn set_size(&mut self, size: usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    extern "C" fn get_size(&mut self, out_size: &mut usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
    // more here but no clue what they are
    extern "C" fn operate_range(&mut self) -> AccessorResult {
        AccessorResult::Unimplemented
    }
}