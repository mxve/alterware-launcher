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

pub fn set_mutex_opt<T: Clone>(cell: &OnceCell<Mutex<T>>, value: Option<T>) {
    if let Some(value) = value {
        set_mutex(cell, value);
    }
}

pub fn get_mutex<T: Clone>(cell: &OnceCell<Mutex<T>>) -> T {
    cell.get()
        .expect("Failed to get once cell value")
        .lock()
        .expect("Failed to lock mutex")
        .clone()
}

pub fn get_mutex_opt<T: Clone>(cell: &OnceCell<Mutex<T>>) -> Option<T> {
    cell.get().map(|m| m.lock().unwrap().clone())
}
