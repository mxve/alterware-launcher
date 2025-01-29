use once_cell::sync::OnceCell;
use std::sync::Mutex;

pub fn set_mutex<T: Clone>(cell: &OnceCell<Mutex<T>>, value: T) {
    if let Some(mutex) = cell.get() {
        if let Ok(mut guard) = mutex.lock() {
            *guard = value;
        }
    } else {
        cell.set(Mutex::new(value)).ok();
    }
}

pub fn get_mutex<T: Clone>(cell: &OnceCell<Mutex<T>>) -> T {
    cell.get()
        .expect("Failed to get once cell value")
        .lock()
        .expect("Failed to lock mutex")
        .clone()
}
