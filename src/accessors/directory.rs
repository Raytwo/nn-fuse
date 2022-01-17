use std::path::PathBuf;
use std::io::Write;

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

#[derive(Copy, Clone)]
pub enum DirectoryEntryType {
    File(i64),
    Directory
}

#[derive(Clone)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub ty: DirectoryEntryType
}

impl DirectoryEntry {
    pub fn new() -> Self {
        DirectoryEntry {
            path: PathBuf::new(),
            ty: DirectoryEntryType::Directory
        }
    }
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
        let mut buf = vec![DirectoryEntry::new(); buffer_len];
        let mut buffer = unsafe {
            std::slice::from_raw_parts_mut(buffer, buffer_len)
        };
        match self.accessor.read(buf.as_mut_slice()) {
            Ok(size) => {
                for (idx, entry) in buf.iter().enumerate() {
                    let mut char_buffer = &mut buffer[idx].name[..];
                    char_buffer.fill(0);
                    let string = entry.path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    char_buffer.write(string.as_bytes());
                    char_buffer[string.len()] = 0;
                    match entry.ty {
                        DirectoryEntryType::Directory => buffer[idx].type_ = 0,
                        DirectoryEntryType::File(size) => {
                            buffer[idx].type_ = 1;
                            buffer[idx].fileSize = size;
                        }
                    }
                }
                *out_count = buf.len() as isize;
            },
            Err(e) => return e,
        }
        AccessorResult::Success
    }

    extern "C" fn get_entry_count(&mut self, out_count: &mut isize) -> AccessorResult {
        match self.accessor.get_entry_count() {
            Ok(size) => {
                *out_count = size as isize;
                AccessorResult::Success
            },
            Err(e) => e
        }
    }
}

pub trait DirectoryAccessor {
    fn read(&mut self, buffer: &mut [DirectoryEntry]) -> Result<usize, AccessorResult>;

    fn get_entry_count(&mut self) -> Result<usize, AccessorResult>;
}