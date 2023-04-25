mod MessageQueue;
use rand::Rng;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread, time,
};
use MessageQueue::MessageQueue as messageQueue;

use crate::MessageQueue::MessageResult;

struct ContextQueue {
    stop: AtomicBool,
    mtx: Mutex<messageQueue<String>>,
}

fn is_non_blocking_pop(rnd: u16) -> bool {
    return (rnd % 3) == 0;
}

fn is_blocking_pop(rnd: u16) -> bool {
    return (rnd % 2) == 0;
}

fn is_blocking_push(rnd: u16) -> bool {
    return (rnd % 1) == 0;
}

fn main() {
    let context = Arc::new(ContextQueue {
        stop: AtomicBool::new(false),
        mtx: Mutex::new(messageQueue::new(3)),
    });
    let mut readers = vec![];
    let mut writers = vec![];
    let num_readers = 3;
    let num_writers = 5;
    for i in 0..num_readers {
        let context_thread = context.clone();
        readers.push(thread::spawn(move || {
            let thread_id = ("reader id".to_string() + &i.to_string()).to_string();
            let sleep_time = rand::thread_rng().gen_range(1..1000);
            thread::sleep(time::Duration::from_millis(sleep_time));
            while !context_thread.stop.load(Ordering::Relaxed) {
                let result: (Option<String>, MessageResult);
                if is_non_blocking_pop(sleep_time.try_into().unwrap()) {
                    result = context_thread.mtx.lock().unwrap().pop::<false>();
                } else if is_blocking_pop(sleep_time.try_into().unwrap()) {
                    result = context_thread.mtx.lock().unwrap().pop::<true>();
                } else {
                    result = context_thread.mtx.lock().unwrap().get(&|message| {
                        return message == "2";
                    });
                }
                if matches!(result.1, MessageResult::Ok) {
                    println!("pop operation succeeded");
                } else {
                    println!("pop operation errored");
                }
            }
        }));
    }
    for i in 0..num_writers {
        let context_thread = context.clone();
        writers.push(thread::spawn(move || {
            let thread_id = ("writer id".to_string() + &i.to_string()).to_string();
            let sleep_time = rand::thread_rng().gen_range(1..1000);
            thread::sleep(time::Duration::from_millis(sleep_time));
            while !context_thread.stop.load(Ordering::Relaxed) {
                let result: MessageResult;
                if is_blocking_push(i) {
                    result = context_thread
                        .mtx
                        .lock()
                        .unwrap()
                        .push::<true>(i.to_string());
                } else {
                    result = context_thread
                        .mtx
                        .lock()
                        .unwrap()
                        .push::<false>(i.to_string());
                }
                if matches!(result, MessageResult::Ok) {
                    println!("push operation succeeded");
                } else {
                    println!("push operation errored");
                }
            }
        }));
    }

    thread::sleep(time::Duration::from_millis(500));
    context.mtx.lock().unwrap().close();
    context.stop.store(true, Ordering::Relaxed);

    for reader in readers {
        if (!reader.is_finished()) {
            reader.join();
        }
    }
    for writer in writers {
        if (!writer.is_finished()) {
            writer.join();
        }
    }

    println!("Program finish!");
}
