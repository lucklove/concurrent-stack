extern crate atomic_stamped_ptr;

use atomic_stamped_ptr::AtomicStampedPtr;

pub struct ConcurrentStack<T> {
    top: AtomicStampedPtr<Node<T>>,
    trash: AtomicStampedPtr<Node<T>>,
}

struct Node<T> {
    data: Option<T>,
    next: *mut Node<T>,
}

impl<T> ConcurrentStack<T> {
    fn put_trash(&self, node: *mut Node<T>) {
        push_top(&self.trash, node);
    }

    fn pick_trash(&self) -> *mut Node<T> {
        pop_top(&self.trash)
    }

    fn do_push(&self, raw: T) {
        let mut node = self.pick_trash();
        if node.is_null() {
            node = Box::into_raw(Box::new(Node {
                data: None,
                next: std::ptr::null_mut(),
            }));
        }
        unsafe {
            (*node).data = Some(raw);
        }
        push_top(&self.top, node);
    }

    pub fn new() -> Self {
        ConcurrentStack {
            top: AtomicStampedPtr::default(),
            trash: AtomicStampedPtr::default(),
        }
    }

    pub fn push(&self, raw: T) {
        self.do_push(raw);
    }

    pub fn pop(&self) -> Option<T> {
        let node = pop_top(&self.top);
        if node.is_null() {
            None
        } else {
            let mut v = None;
            std::mem::swap(&mut v, unsafe { &mut (*node).data });
            self.put_trash(node);
            v
        }
    }

    pub fn empty(&self) -> bool {
        self.top.load().0.is_null()
    }
}

impl<T> Drop for ConcurrentStack<T> {
    fn drop(&mut self) {
        release(&self.top);
        release(&self.trash);
    }
}

fn push_top<T>(top: &AtomicStampedPtr<Node<T>>, node: *mut Node<T>) {
    loop {
        let (p, v) = top.load();
        unsafe {
            (*node).next = p;
        }
        if let Ok(_) = top.compare_exchange((p, v), node) {
            break;
        }
    }
}

fn pop_top<T>(top: &AtomicStampedPtr<Node<T>>) -> *mut Node<T> {
    loop {
        let (p, v) = top.load();
        if p.is_null() {
            return p;
        }
        let n = unsafe { (*p).next };
        if let Ok(_) = top.compare_exchange((p, v), n) {
            return p;
        }
    }
}

fn release<T>(top: &AtomicStampedPtr<Node<T>>) {
    let (mut p, _) = top.load();
    while !p.is_null() {
        let d = p;
        unsafe {
            p = (*p).next;
            drop(Box::from_raw(d));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConcurrentStack;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn lock_free_stack_single_thread() {
        let stack = ConcurrentStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        stack.push(4);
        assert_eq!(stack.pop(), Some(4));
        assert_eq!(stack.pop(), Some(1));
    }

    #[test]
    fn multi_thread_sum() {
        let stack = Arc::new(ConcurrentStack::new());

        let input_p = (0..10).map(|_| {
            let stack = stack.clone();
            thread::spawn(move || {
                for i in 0..100 {
                    stack.push(i);
                }
            })
        }).collect::<Vec<_>>();


        let mut sum = 0;
         
        let output_p = {
            let stack = stack.clone();
            thread::spawn(move || {
                loop {
                    if let Some(i) = stack.pop() {
                        sum += i;
                    } else if sum == 49500 {
                        break;
                    }
                }
            })
        };

        for t in input_p {
            t.join().unwrap();
        }
        output_p.join().unwrap();

        assert!(stack.empty());
    }

    #[test]
    fn store_uncopyable() {
        let stack = ConcurrentStack::new();
        stack.push(Box::new(1));
        stack.push(Box::new(2));
        stack.push(Box::new(3));
        assert_eq!(*stack.pop().unwrap(), 3);
        assert_eq!(*stack.pop().unwrap(), 2);
        assert_eq!(*stack.pop().unwrap(), 1);
    }
}
