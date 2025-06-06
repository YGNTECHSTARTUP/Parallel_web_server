use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Condvar, Mutex},
    thread,
};
#[derive(Debug, Default)]
pub struct Cache<K, V> {
    map: Mutex<HashMap<K, Arc<Entry<V>>>>,
}

#[derive(Debug)]
enum EntryState<V> {
    Computing,
    Ready(V),
}

#[derive(Debug)]
struct Entry<V> {
    state: Mutex<EntryState<V>>,
    cvar: Condvar,
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
    pub fn get_or_insert_with<F: FnOnce(K) -> V>(&self, key: K, f: F) -> V {
        let entry = {
            let mut map_guard = self.map.lock().unwrap();
            match map_guard.get(&key) {
                Some(v) => Arc::clone(v),
                None => {
                    let entry = Arc::new(Entry {
                        state: Mutex::new(EntryState::Computing),
                        cvar: Condvar::new(),
                    });
                    map_guard.insert(key.clone(), Arc::clone(&entry));
                    entry
                }
            }
        };
        let mut entry_lock = entry.state.lock().unwrap();
        match &*entry_lock {
            EntryState::Computing => {
                let if_first = Arc::strong_count(&entry) == 2;
                if if_first {
                    drop(entry_lock);
                    let value = f(key.clone());
                    let mut entry_lock = entry.state.lock().unwrap();
                    *entry_lock = EntryState::Ready(value.clone());
                    entry.cvar.notify_all();
                    return value;
                } else {
                    loop {
                        match &*entry_lock {
                            EntryState::Computing => {
                                entry_lock = entry.cvar.wait(entry_lock).unwrap();
                            }
                            EntryState::Ready(v) => return v.clone(),
                        }
                    }
                }
            }
            EntryState::Ready(v) => {
                return v.clone();
            }
        }
    }
}

pub fn expensive_fib(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => expensive_fib(n - 1) + expensive_fib(n - 2),
    }
}

pub fn cha() {
    let cache: Arc<Cache<u32, u64>> = Arc::new(Cache::new());
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || {
                let result = cache.get_or_insert_with(10, |n| expensive_fib(n));
                println!("Got fib 20 = {result}");
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}
