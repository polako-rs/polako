use std::{sync::RwLock, thread::ThreadId, rc::Rc, cell::RefCell};

use bevy::{ecs::system::Resource, utils::HashMap};

#[derive(Resource)]
pub struct Channel<T>(RwLock<HashMap<ThreadId, Rc<RefCell<Vec<T>>>>>);
unsafe impl<T> Send for Channel<T> {}
unsafe impl<T> Sync for Channel<T> {}
impl<T> Channel<T> {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub fn clear(&self) {
        for cell in self.0.read().unwrap().values() {
            cell.borrow_mut().clear()
        }
    }
    pub fn send(&self, event: T) {
        let id = std::thread::current().id();
        {
            let read = self.0.read().unwrap();
            let item = read.get(&id);
            if let Some(events) = item {
                events.borrow_mut().push(event);
                return;
            }
        }
        {
            self.0
                .write()
                .unwrap()
                .insert(id, Rc::new(RefCell::new(vec![event])));
        }
    }
    pub fn recv<F: FnMut(&T)>(&self, mut recv: F) {
        for cell in self.0.read().unwrap().values() {
            let borrow = cell.borrow();
            for item in borrow.iter() {
                recv(item)
            }
        }
    }

    pub fn consume<F: FnMut(T)>(&mut self, mut recv: F) {
        for cell in self.0.write().unwrap().drain() {
            for item in cell.1.take() {
                recv(item)
            }
        }
    }
}