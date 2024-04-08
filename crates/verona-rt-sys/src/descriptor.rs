//! Object Descriptor
//!
//! See `src/rt/object/object.h` in `verona-rt`

use core::marker::{PhantomData, PhantomPinned};

#[repr(C)]
// TODO: Is this FFI Safe??
pub struct Object {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
    // TODO: Give unique_id with USE_SYSTEMATIC_TESTING
}

#[repr(C)]
pub struct ObjectStack {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

pub type TraceFunction = extern "C" fn(o: *const Object, st: *mut ObjectStack);

pub type NotifiedFunction = extern "C" fn(o: *mut Object);

pub type FinalFunction = extern "C" fn(o: *mut Object, region: *mut Object, st: *mut ObjectStack);

pub type DestructorFunction = extern "C" fn(o: *mut Object);

pub extern "C" fn noop_trace(_o: *const Object, _st: *mut ObjectStack) {}

#[repr(C)]
// TODO: Make sure this is aligned correctly on non-64bit platforms.
pub struct Descriptor {
    pub size: usize,
    pub trace: TraceFunction,
    pub finaliser: Option<FinalFunction>,
    pub notified: Option<NotifiedFunction>,
    pub destructor: Option<DestructorFunction>,
}

#[test]
fn size_and_align() {
    #[link(name = "boxcar_bindings")]
    extern "C" {
        fn boxcars_test_descriptor_info(size: &mut usize, align: &mut usize);
    }
    let mut size = 0;
    let mut align = 0;
    unsafe {
        boxcars_test_descriptor_info(&mut size, &mut align);
    }
    assert_eq!(size, std::mem::size_of::<Descriptor>());
    assert_eq!(align, std::mem::align_of::<Descriptor>())
}

#[test]
fn noop_signature() {
    let _: TraceFunction = noop_trace;
}
