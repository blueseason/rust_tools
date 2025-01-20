use std::cell::{RefCell, RefMut};

use lazy_static::*;
pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Exclusive access inner data in UPSafeCell. Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.inner.as_ptr()
    }
}

fn main() {
    lazy_static! {
        static ref MEMORY_POOL: UPSafeCell<[u8; 4096]> = unsafe { UPSafeCell::new([0; 4096]) };
    }
    let mem_ptr = MEMORY_POOL.as_ptr();
    println!(
        "the memory pool UPSafeCell ptr is {:p}",
        &MEMORY_POOL.exclusive_access()
    );

    println!("the memory pool raw ptr is {:p}", mem_ptr);
    println!("the memory pool raw ptr usize is 0x{:x}", mem_ptr as usize);

    let data = RefCell::new(42);
    unsafe {
        // cast to const raw ptr use borrow
        let _data_ptr: *const i32 = &*data.borrow();

        // as ptr can get the raw mut ptr
        let data_ptr = data.as_ptr();

        *data_ptr = 1;
        println!("Value through const pointer: {}", *data_ptr);
    }

    // cast to mut raw ptr
    let mut_ptr: *mut i32 = &mut *data.borrow_mut();
    unsafe {
        *mut_ptr = 50; // 修改值
        println!("Value through mut pointer: {}", *mut_ptr);
    }
}
