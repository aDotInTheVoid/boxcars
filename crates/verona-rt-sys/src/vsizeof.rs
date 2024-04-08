use std::mem::size_of;

// Only used for size calculations.
#[allow(dead_code)]
#[repr(C)]
#[cfg_attr(target_pointer_width = "64", repr(align(16)))]
struct ObjectHeader {
    f1: *mut (),
    f2: *mut (),
    #[cfg(feature = "systematic_testing")]
    f3: *mut (),
}

const SIZEOF_OBJECT_HEADER: usize = size_of::<ObjectHeader>();
const OBJECT_ALIGNMENT: usize = std::mem::align_of::<ObjectHeader>();

pub const fn vsizeof<T>() -> usize {
    // The runtime stores an object header below the returned pointer, but we still need space for it in the allocation.
    align_up(size_of::<T>() + SIZEOF_OBJECT_HEADER, OBJECT_ALIGNMENT)
}

const fn align_up(value: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());
    let align_1 = alignment - 1;
    return (value + align_1) & !align_1;
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" {
        fn boxcar_vsizeof_info(sizeof_object_header: &mut usize, object_alignment: &mut usize);
        fn boxcar_vsizeof_examples(
            bool: &mut usize,
            i32: &mut usize,
            voidstar: &mut usize,
            charx17: &mut usize,
        );
    }

    #[test]
    fn metadata() {
        let mut sizeof_object_header = 0;
        let mut object_alignment = 0;

        unsafe {
            boxcar_vsizeof_info(&mut sizeof_object_header, &mut object_alignment);
        }

        assert_eq!(sizeof_object_header, SIZEOF_OBJECT_HEADER);
        assert_eq!(object_alignment, OBJECT_ALIGNMENT)
    }

    #[test]
    fn examples() {
        let mut bool_ = 0;
        let mut i32_ = 0;
        let mut voidstar = 0;
        let mut charx17_ = 0;

        unsafe { boxcar_vsizeof_examples(&mut bool_, &mut i32_, &mut voidstar, &mut charx17_) }

        assert_eq!(bool_, vsizeof::<bool>());
        assert_eq!(i32_, vsizeof::<i32>());
        assert_eq!(voidstar, vsizeof::<*mut ()>());
        assert_eq!(charx17_, vsizeof::<[u8; 17]>());
    }
}
