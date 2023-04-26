use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

pub enum MessageResult {
    Ok,
    Empty,
    Full,
    NotFound,
    Closed,
}

pub struct MessageQueue<Message> {
    m_mtx: Mutex<Vec<Message>>,
    m_pop_cvr: Condvar,
    m_push_cvr: Condvar,
    m_queue_size: usize,
    //for no blocking while check state
    m_state_closed: AtomicBool,
}

impl<Message> MessageQueue<Message> {
    pub fn new(queue_size: usize) -> Self {
        Self {
            m_mtx: Mutex::new(vec![]),
            m_pop_cvr: Condvar::new(),
            m_push_cvr: Condvar::new(),
            m_queue_size: queue_size,
            m_state_closed: AtomicBool::new(false),
        }
    }

    pub fn push<const POLICY: bool>(&self, message: Message) -> MessageResult {
        if self.is_closed() {
            return MessageResult::Closed;
        }
        {
            let mut lk = self.m_mtx.lock().unwrap();
            // let lk = &mut *self.m_mtx.lock().unwrap();
            if (lk).len() == self.m_queue_size {
                if matches!(POLICY, false) {
                    return MessageResult::Full;
                } else {
                    //assert!();
                    // let y = &mut *lk;
                    lk = self
                        .m_push_cvr
                        .wait_while(lk, move |pending: &mut Vec<Message>| {
                            return self.is_closed() || !(&*pending).len() < self.m_queue_size;
                        })
                        .unwrap();
                    if self.is_closed() {
                        return MessageResult::Closed;
                    }
                }
            }
            (lk).push(message);
        }
        self.m_pop_cvr.notify_one();

        return MessageResult::Ok;
    }

    pub fn pop<const POLICY: bool>(&self) -> (Option<Message>, MessageResult) {
        if self.is_closed() {
            return (None, MessageResult::Closed);
        }

        let msg: Option<Message>;
        {
            //unique lock
            let mut lk = self.m_mtx.lock().unwrap();
            if (*lk).is_empty() {
                if matches!(POLICY, false) {
                    return (None, MessageResult::Empty);
                } else {
                    //assert!();
                    lk = self
                        .m_pop_cvr
                        .wait_while(lk, move |pending: &mut Vec<Message>| {
                            return self.is_closed() || !(&*pending).is_empty();
                        })
                        .unwrap();
                    if self.is_closed() {
                        return (None, MessageResult::Closed);
                    }
                }
            }
            msg = (*lk).pop();
            //self.m_queue.pop();
        }
        self.m_push_cvr.notify_one();
        return (msg, MessageResult::Ok);
    }

    pub fn get(&self, predicate: &dyn Fn(&Message) -> bool) -> (Option<Message>, MessageResult) {
        if self.is_closed() {
            return (None, MessageResult::Closed);
        }

        let mut msg: Option<Message> = None;
        {
            let mut lk = self.m_mtx.lock().unwrap();

            let index_elem = (*lk).iter().position(predicate);
            if index_elem.is_some() {
                msg = Some((*lk).swap_remove(index_elem.unwrap()));
            }
            else{
                return (None,MessageResult::NotFound);
            }
        }
        self.m_push_cvr.notify_one();
        return (msg, MessageResult::Ok);
    }

    pub fn close(&self) -> MessageResult {
        self.m_state_closed.store(true, Ordering::Relaxed);
        self.m_pop_cvr.notify_all();
        self.m_push_cvr.notify_all();
        return MessageResult::Ok;
    }

    pub fn is_closed(&self) -> bool {
        return self.m_state_closed.load(Ordering::Relaxed);
    }
}
