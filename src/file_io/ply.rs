use crate::math::vector::Vec3;

use rply;

use std::mem::MaybeUninit;
use std::os::raw;
use std::ptr;

fn error_cb(ply: rply::p_ply, message: *const raw::c_char) {}

fn vec3_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = MaybeUninit::uninit().assume_init();
        let mut buffer_ptr = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        (item_index as usize, &mut *(buffer_ptr as *mut Vec<f32>))
    };

    let index = unsafe {
        let mut index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    let buff_index = index + 3 * item_index;
    unsafe {
        *buffer.get_unchecked_mut(buff_index) = rply::ply_get_argument_value(argument) as f32;
    }

    1
}

fn vec2_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = MaybeUninit::uninit().assume_init();
        let mut buffer_ptr = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        (item_index as usize, &mut *(buffer_ptr as *mut Vec<f32>))
    };

    let index = unsafe {
        let mut index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    let buff_index = index + 2 * item_index;
    unsafe {
        *buffer.get_unchecked_mut(buff_index) = rply::ply_get_argument_value(argument) as f32;
    }

    1
}

enum IndexProcStatus {
    NO_ISSUE,      // Nothing wrong happened.
    INC_FACE_SIZE, // i.e. If it's a triangle and there are more than 3 points.
    NEG_INDEX,     // If a negative index was passed.
}

struct IndexBuffer {
    proc_status: IndexProcStatus,
    face_count: usize,
    buffer: Vec<u32>,
}

fn index_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let buffer = unsafe {
        let mut buffer_ptr = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, ptr::null_mut()) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        &mut *(buffer_ptr as *mut IndexBuffer)
    };

    let (num_indices, face_index) = unsafe {
        let mut num_indices = MaybeUninit::uninit().assume_init();
        let mut face_index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_property(
            argument,
            ptr::null_mut(),
            &mut num_indices,
            &mut face_index,
        ) == 0
        {
            return 0;
        }
        (num_indices as usize, face_index as usize)
    };

    if num_indices != buffer.face_count {
        buffer.proc_status = IndexProcStatus::INC_FACE_SIZE;
        return 0;
    }

    if face_index < 0 {
        buffer.proc_status = IndexProcStatus::NEG_INDEX;
        return 0;
    }

    let index = unsafe {
        let mut index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    let buff_index = face_index + num_indices * index;
    unsafe {
        *buffer.buffer.get_unchecked_mut(buff_index) =
            rply::ply_get_argument_value(argument) as u32;
    }

    1
}
