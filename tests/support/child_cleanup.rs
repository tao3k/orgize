use std::process::Child;

// Parent-owned handles permit cleanup even if a child never writes to a pipe.
pub(super) struct Children(pub(super) Vec<Child>);

impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
        }
        for child in &mut self.0 {
            let _ = child.wait();
        }
    }
}
