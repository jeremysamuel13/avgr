use parking_lot::Mutex;

pub struct State<T> {
    pub state: Mutex<T>,
}
