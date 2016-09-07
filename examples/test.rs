extern crate concurrent_stack;

use concurrent_stack::ConcurrentStack;

fn main() {
    let stack = ConcurrentStack::new();
    stack.push(Box::new(1));
    stack.push(Box::new(2));
    stack.push(Box::new(3));
}