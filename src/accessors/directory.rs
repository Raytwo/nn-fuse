use crate::{ fs, AccessorResult };

use skyline::nn;

#[repr(C)]
struct DirectoryAccessorVtable {
    // also type info at VTable - 0x8
    destructor: extern "C" fn(&mut DAccessor),
    deleter: extern "C" fn(&mut DAccessor),
    read: extern "C" fn(&mut DAccessor, &mut isize, *mut nn::fs::DirectoryEntry, usize) -> AccessorResult,
    get_entry_count: extern "C" fn(&mut DAccessor, &mut isize) -> AccessorResult
}

static DACCESSOR_VTABLE: DirectoryAccessorVtable = DirectoryAccessorVtable {
    destructor: DAccessor::destructor,
    deleter: DAccessor::deleter,
    read: DAccessor::read,
    get_entry_count: DAccessor::get_entry_count,
};

#[repr(C)]
pub struct DAccessor {
    vtable: &'static DirectoryAccessorVtable,
    accessor: Box<dyn DirectoryAccessor>,
}


impl DAccessor {
    pub fn new<D: DirectoryAccessor + 'static>(accessor: D) -> *mut Self {
        let mut out = fs::detail::alloc::<Self>();

        // SAFETY: Do not change this way of assigning the values in case of refactoring. Dereferencing `out` to assign a new instance of the struct would call the destructor for the Box field, which is uninitialized, and cause a crash.
        unsafe {
            out.write(Self {
                vtable: &DACCESSOR_VTABLE,
                accessor: Box::new(accessor) as _,
            });
        }

        let out: *mut DAccessor = unsafe { std::mem::transmute(out) };
        out
    }

    extern "C" fn destructor(&mut self) { }

    extern "C" fn deleter(&mut self) {
        self.destructor();
        fs::detail::free(self);
    }

    extern "C" fn read(&mut self, out_count: &mut isize, buffer: *mut nn::fs::DirectoryEntry, buffer_len: usize) -> AccessorResult {
        AccessorResult::Unimplemented
    }

    extern "C" fn get_entry_count(&mut self, out_count: &mut isize) -> AccessorResult {
        AccessorResult::Unimplemented
    }
}

pub trait DirectoryAccessor {
}