// A bunch of useful functions when dealing with memory:

// Reinterprets the memory of a vector to that of another:
pub unsafe fn transmute_vec<U, T>(mut src: Vec<U>) -> Vec<T> {
    // First we extract everything we want:
    let src_ptr = src.as_mut_ptr();
    let src_len = src.len();
    let src_cap = src.capacity();

    // Get the size of the information:
    let size_u = std::mem::size_of::<U>();
    let size_t = std::mem::size_of::<T>();

    // Get new length required here:
    let src_len = (src_len * size_u) / size_t;

    // "Forget" src so that we don't call the destructor on src (which would delete our memory)
    std::mem::forget(src);
    let src_ptr = std::mem::transmute::<*mut U, *mut T>(src_ptr);

    // Construct the new vector:
    Vec::from_raw_parts(src_ptr, src_len, src_cap)
}

// Allocates an array of UNINITIALIZED data. Not the most efficient
// thing in the world (probably). I'll have to look into it.
pub unsafe fn alloc_array<T: Sized>(len: usize) -> Box<[T]> {
    // Allocate the space using vector (I know I know...)
    let mut array = Vec::with_capacity(len);
    array.set_len(len);
    array.into_boxed_slice()
}
