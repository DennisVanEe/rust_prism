// The stack allocator is an allocator designed to efficiently allocate memory with few calls to new.
// It is essentialy a bunch of stacks linked by a linked list.
// It uses a bunch of unsafe code because it's suppose to be as fast as possible.

use crate::util::alloc_array;

// This only works if 
pub struct StackAlloc<T: Sized> {
    stack_size: usize,
    curr_pos: usize, // current position into the stack that we are working with
    alloc_count: usize, // keeps track of how much we allocated so far.
    data: Vec<Box<[T]>>,
}

impl<T: Sized> StackAlloc<T> {
    pub fn new(stack_size: usize) -> Self {
        // This is safe, because we manage this ourselves:
        StackAlloc {
            stack_size,
            curr_pos: 0,
            alloc_count: 0,
            data: unsafe { vec![alloc_array(stack_size)] },
        }
    }

    // Moves the entry into memory so that it exists in space somewhere, always:
    pub fn push<'a>(&'a mut self, value: T) -> &'a mut T {
        // Check if we should allocate more memory:
        if self.curr_pos == self.stack_size {
            // Allocate the new array and push it:
            unsafe { self.data.push(alloc_array(self.stack_size)) };
            self.curr_pos = 0;
        }

        // Get the current box we care about:
        let curr_stack = {
            let last_index = self.data.len() - 1;
            // We are guaranteed that data is at least length 1:
            unsafe { self.data.get_unchecked_mut(last_index) }
        };

        // Set the value and get a reference to it:
        let result = unsafe { 
            *curr_stack.get_unchecked_mut(self.curr_pos) = value;
            curr_stack.get_unchecked_mut(self.curr_pos)
        };
        self.curr_pos += 1;
        self.alloc_count += 1;
        result
    }

    // Deallocates all of the memory, still need to allocate a stack:
    pub fn clear(&mut self, stack_size: usize) {
        self.stack_size = stack_size;
        self.curr_pos = 0;
        self.alloc_count = 0;
        self.data = unsafe{ vec![alloc_array(stack_size)] };
    }

    pub fn get_num_alloc(&self) -> usize {
        self.alloc_count
    }
}