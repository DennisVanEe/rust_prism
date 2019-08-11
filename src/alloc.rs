// The stack allocator is an allocator designed to efficiently allocate memory with few calls to new.
// It is essentialy a bunch of stacks linked by a linked list.
// It uses a bunch of unsafe code because it's suppose to be as fast as possible.

use crate::util::alloc_array;
use std::cell::{Cell, UnsafeCell};

// This only works with copyable data for now so we don't have to worry about dropping:
pub struct StackAlloc<T: Copy> {
    curr_pos: Cell<usize>, // Current position into the stack that we are working with
    count: Cell<usize>,    // Keeps track of what we stored onto the stack so far.
    data: UnsafeCell<Vec<Box<[T]>>>, // The actual data itself resides here.
}

impl<T: Copy> StackAlloc<T> {
    pub fn new(stack_size: usize) -> Self {
        // This is safe, because we manage this ourselves:
        StackAlloc {
            curr_pos: Cell::new(0),
            count: Cell::new(0),
            data: unsafe { UnsafeCell::new(vec![alloc_array(stack_size)]) },
        }
    }

    // We make it an immutable borrow so we can pass it arround easier without
    // the borrow checker going crazy (see use in BVH).
    pub fn push(&self, value: T) -> &T {
        // Check if we should allocate more memory:
        let stacks = unsafe { &mut *self.data.get() };
        let stacks_end_index = stacks.len() - 1;

        let stack_len = unsafe { stacks.get_unchecked(stacks_end_index).len() };

        // Check if we should deallocate:
        if self.curr_pos.get() == stack_len {
            // Allocate the new array and push it:
            unsafe { stacks.push(alloc_array(stack_len)) };
            self.curr_pos.set(0);
        }

        // Get the current box we care about.
        // We are guaranteed that data is at least length 1:
        let curr_stack = unsafe { stacks.get_unchecked_mut(stacks_end_index) };

        // Set the value and get a reference to it:
        let result = unsafe { curr_stack.get_unchecked_mut(self.curr_pos.get()) };
        *result = value;
        self.curr_pos.set(self.curr_pos.get() + 1);
        self.count.set(self.count.get() + 1);
        result
    }

    pub fn get_alloc_count(&self) -> usize {
        self.count.get()
    }
}
