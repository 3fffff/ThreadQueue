use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

/*pub enum OperationPolicy {
    Blocking,
    Nonblocking,
}
trait OpPolicy/*  for OperationPolicy*/{
    fn as_match(&self) -> OperationPolicy;
}

impl  OpPolicy for OperationPolicy {
    fn as_match(&self) -> OperationPolicy {
        match self {
            OperationPolicy::One => OperationPolicy::One,
            OperationPolicy::Two => OperationPolicy::Two,
        }
    }
}*/

pub enum MessageResult {
    Ok,
    Empty,
    Full,
    NotFound,
    Closed,
}

pub struct MessageQueue<Message> {
    m_mtx: Mutex<Vec<Message>>,
    m_PopCv: Condvar,
    m_pushCv: Condvar,
    m_queue_size: usize,
    //for no blocking while check state
    m_state: AtomicBool,
    //PolicyBlocking: bool,
}

impl<Message> MessageQueue<Message> {
    pub fn new(queue_size: usize) -> Self {
        Self {
            m_mtx: Mutex::new(vec![]),
            m_PopCv: Condvar::new(),
            m_pushCv: Condvar::new(),
            m_queue_size: queue_size,
            m_state: AtomicBool::new(false),
            //PolicyBlocking:false,
        }
    }

    pub fn push<const policy: bool>(&self, message: Message) -> MessageResult {
        if self.is_closed() {
            return MessageResult::Closed;
        }
        {
            let mut lk = self.m_mtx.lock().unwrap();
            // let lk = &mut *self.m_mtx.lock().unwrap();
            if (lk).len() == self.m_queue_size {
                if matches!(policy,false) {
                    return MessageResult::Full;
                } else {
                    //assert!();
                    // let y = &mut *lk;
                    lk = self
                        .m_pushCv
                        .wait_while(lk, move |pending: &mut Vec<Message>| {
                            return self.is_closed() || !(&*pending).is_empty();
                        })
                        .unwrap();
                    if self.is_closed() {
                        return MessageResult::Closed;
                    }
                }
            }
            //let mut lk_push = self.m_mtx.lock().unwrap();
            (lk).push(message);
        }
        self.m_PopCv.notify_one();

        return MessageResult::Ok;
    }

    pub fn pop<const policy:bool>(&self) -> (Option<Message>, MessageResult)
    {
        if self.is_closed() {
            return (None, MessageResult::Closed);
        }

        let msg: Message;
        {
            //unique lock
            let mut lk = self.m_mtx.lock().unwrap();
            if (*lk).len() == self.m_queue_size {
                if matches!(policy, false) {
                    return (None, MessageResult::Full);
                } else {
                    //assert!();
                    lk = self
                        .m_PopCv
                        .wait_while(lk, move |pending: &mut Vec<Message>| {
                            return self.is_closed() || !(&*pending).is_empty();
                        })
                        .unwrap();
                    if self.is_closed() {
                        return (None, MessageResult::Closed);
                    }
                }
            }
            msg = (*lk).pop().unwrap();
            //self.m_queue.pop();
        }
        self.m_pushCv.notify_one();
        return (Some(msg), MessageResult::Ok);
    }

    pub fn get(&self, predicate: &dyn Fn(&Message) -> bool) -> (Option<Message>, MessageResult) {
        if self.is_closed() {
            return (None, MessageResult::Closed);
        }

        let msg: Option<Message>;
        {
            let mut lk = self.m_mtx.lock().unwrap();

            let index_elem = ((*lk).iter().position(predicate)).unwrap();
            msg = Some((*lk).swap_remove(index_elem));
        }
        self.m_pushCv.notify_one();
        return (msg, MessageResult::Ok);
    }

    pub fn close(&self) -> MessageResult {
        self.m_state.store(true, Ordering::Relaxed);
        self.m_PopCv.notify_all();
        self.m_pushCv.notify_all();
        return MessageResult::Ok;
    }

    pub fn is_closed(&self) -> bool {
        return self.m_state.load(Ordering::Relaxed);
    }
}
