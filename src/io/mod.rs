//! Traits and something else for asynchronous I/O operations.

pub unsafe trait EmmaBuf: Unpin + 'static {
    fn ptr(&self) -> *const u8;
    fn mut_ptr(&mut self) -> *mut u8;
    fn bytes(&self) -> usize;
}

unsafe impl EmmaBuf for Vec<u8> {
    fn ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
    fn bytes(&self) -> usize {
        self.capacity()
    }
}

unsafe impl EmmaBuf for Box<[u8]> {
    fn ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
    fn bytes(&self) -> usize {
        self.len()
    }
}

// todo
