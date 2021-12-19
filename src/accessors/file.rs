use crate::{ fs, AccessorResult };

use skyline::nn;

#[repr(C)]
struct FileAccessorVtable {
    destructor: extern "C" fn(&mut FAccessor),
    deleter: extern "C" fn(&mut FAccessor),
    read: extern "C" fn(&mut FAccessor, &mut usize, usize, *mut u8, usize, u32) -> AccessorResult,
    write: extern "C" fn(&mut FAccessor, usize, *const u8, usize, &nn::fs::WriteOption) -> AccessorResult,
    flush: extern "C" fn(&mut FAccessor) -> AccessorResult,
    set_size: extern "C" fn(&mut FAccessor, usize) -> AccessorResult,
    get_size: extern "C" fn(&mut FAccessor, &mut usize) -> AccessorResult,
    // more here but no clue what they are
    operate_range: extern "C" fn(&mut FAccessor, ) -> AccessorResult
}

static FACCESSOR_VTABLE: FileAccessorVtable  = FileAccessorVtable {
    destructor: FAccessor::destructor,
    deleter: FAccessor::deleter,
    read: FAccessor::read,
    write: FAccessor::write,
    flush: FAccessor::flush,
    set_size: FAccessor::set_size,
    get_size: FAccessor::get_size,
    operate_range: FAccessor::operate_range
};

#[repr(C)]
pub struct FAccessor {
    vtable: &'static FileAccessorVtable,
    options: nn::fs::OpenMode,
    accessor: Box<dyn FileAccessor>,
}

impl FAccessor {
    pub fn new<F: FileAccessor + 'static>(accessor: F, options: nn::fs::OpenMode) -> *mut Self {
        let mut out = fs::detail::alloc::<Self>();

        // SAFETY: Do not change this way of assigning the values in case of refactoring. Dereferencing `out` to assign a new instance of the struct would call the destructor for the Box field, which is uninitialized, and cause a crash.
        unsafe {
            out.write(Self {
                vtable: &FACCESSOR_VTABLE,
                options,
                accessor: Box::new(accessor) as _,
            });
        }

        let out: *mut FAccessor = unsafe { std::mem::transmute(out) };
        out
    }

    extern "C" fn destructor(&mut self) { }

    extern "C" fn deleter(&mut self) {
        self.destructor();
        fs::detail::free(self);
    }

    extern "C" fn read(&mut self, read_size: &mut usize, offset: usize, buffer: *mut u8, buffer_len: usize, read_options: u32) -> AccessorResult {
        println!("FAccessor::read");
        let buffer = unsafe { std::slice::from_raw_parts_mut(buffer, buffer_len) };
        
        match self.accessor.read(buffer, offset) {
            Ok(size) => {
                *read_size = size;
                AccessorResult::Success
            },
            Err(e) => e
        }
    }

    extern "C" fn write(&mut self, offset: usize, data: *const u8, data_len: usize, write_options: &nn::fs::WriteOption) -> AccessorResult {
        println!("FAccessor::write");

        let data = unsafe {
            std::slice::from_raw_parts(data, data_len)
        };

        match self.accessor.write(data, offset, true) {
            Ok(_) => AccessorResult::Success,
            Err(e) => e
        }
    }

    extern "C" fn flush(&mut self) -> AccessorResult {
        println!("FAccessor::flush");

        self.accessor.flush()
    }

    extern "C" fn set_size(&mut self, new_size: usize) -> AccessorResult {
        println!("FAccessor::set_size");

        match self.accessor.set_size(new_size) {
            Ok(()) => AccessorResult::Success,
            Err(e) => e
        }
    }

    extern "C" fn get_size(&mut self, out_size: &mut usize) -> AccessorResult {
        println!("FAccessor::get_size");

        match unsafe { (*self.accessor).get_size() } {
            Ok(size) => {
                *out_size = size;
                AccessorResult::Success
            },
            Err(e) => e
        }
    }

    extern "C" fn operate_range(&mut self, /* ... */) -> AccessorResult {
        panic!("FAccessor");

        AccessorResult::Unsupported
    }
}

pub trait FileAccessor {
    fn read(&mut self, buffer: &mut [u8], offset: usize) -> Result<usize, AccessorResult>;

    fn write(&mut self, data: &[u8], offset: usize, should_append: bool) -> Result<(), AccessorResult> {
        Err(AccessorResult::Unsupported)
    }

    fn set_size(&mut self, new_size: usize) -> Result<(), AccessorResult> {
        Err(AccessorResult::Unsupported)
    }

    fn get_size(&mut self) -> Result<usize, AccessorResult>;

    fn flush(&mut self) -> AccessorResult {
        AccessorResult::Unsupported
    }
}