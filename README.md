Lock free stack
===============

A thread safe FILO structure.

### Basic Usage:
```rust
use concurrent_stack::ConcurrentStack;
use std::sync::Arc;
use std::thread;

let stack = Arc::new(ConcurrentStack::new());
let pusher = stack.clone();
let producer = thread::spawn(move || {
    for i in 0..100 {
        pusher.push(i);
    }
});
let poper = stack.clone();
let consumer = thread::spawn(move || {
    for _ in 0..100 {
        if let Some(v) = poper.pop() {
            // Deal with v.
        }
    }   
});
producer.join();
consumer.join();
```