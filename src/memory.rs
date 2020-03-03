// A bunch of useful functions when dealing with memory:

use std::mem;
use std::ptr;

// Reinterprets the memory of a vector to that of another:
pub unsafe fn transmute_vec<U, T>(mut src: Vec<U>) -> Vec<T> {
    // First we extract everything we want:
    let src_ptr = src.as_mut_ptr();
    let src_len = src.len();
    let src_cap = src.capacity();

    // Get the size of the information:
    let size_u = mem::size_of::<U>();
    let size_t = mem::size_of::<T>();

    // Get new length required here:
    let src_len = (src_len * size_u) / size_t;

    // "Forget" src so that we don't call the destructor on src (which would delete our memory)
    std::mem::forget(src);
    let src_ptr = mem::transmute::<*mut U, *mut T>(src_ptr);

    // Construct the new vector:
    Vec::from_raw_parts(src_ptr, src_len, src_cap)
}

// Allocates a vector of uninitialized data:
pub unsafe fn uninit_vec<T>(size: usize) -> Vec<T> {
    let mut vec = Vec::with_capacity(size);
    vec.set_len(size);
    vec
}

// Allows different types to have their pointers compared:
pub fn is_ptr_same<T0: ?Sized, T1: ?Sized>(a: &T0, b: &T1) -> bool {
    unsafe {
        let bptr = b as *const T1;
        let bptr: *const T0 = mem::transmute_copy(&bptr);
        ptr::eq(a, bptr)
    }
}
