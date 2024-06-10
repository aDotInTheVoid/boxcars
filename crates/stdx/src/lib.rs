use std::sync::{Arc, Mutex};

pub mod rand;
pub mod thread;

pub struct SetOnDrop(pub Arc<Mutex<bool>>);
impl std::ops::Drop for SetOnDrop {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }

        let mut is_droped = self.0.lock().unwrap();
        assert_eq!(*is_droped, false);
        *is_droped = true;
    }
}
impl SetOnDrop {
    pub fn new() -> (Self, Arc<Mutex<bool>>) {
        let state: Arc<Mutex<bool>> = Arc::default();
        (Self(state.clone()), state)
    }
}
