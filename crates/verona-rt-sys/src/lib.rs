#[repr(C)]
struct CownPtr(*const ());

#[link(name = "boxcar_bindings")]
extern "C" {
    fn make_account(_: i32, _: bool) -> CownPtr;

    // fn lol();

    fn zzadd(_: i32, _: i32) -> i32;
}

#[test]
fn call_funcs() {
    unsafe {
        assert_eq!(zzadd(1, 2), 3);

        let x = make_account(1, false);
        // make_account(0, false);
    }
}
