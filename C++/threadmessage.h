#ifndef THREAD_MESSAGE_H_
#define THREAD_MESSAGE_H_

#include <algorithm>
#include <atomic>
#include <condition_variable>
#include <vector>
#include <mutex>
#include <stdexcept>
#include <utility>

template<typename Message>
class ThreadMessage final {
    ThreadMessage(const ThreadMessage&) = delete;
    ThreadMessage(ThreadMessage&&) = delete;
    ThreadMessage& operator=(const ThreadMessage&) = delete;
    ThreadMessage& operator=(ThreadMessage&&) = delete;

    enum class MessageResult {
        Ok,
        Empty,
        Full,
        NotFound,
        Closed,
    };
public:
    ThreadMessage(std::size_t max_queque_size) :max_queue_size(max_queue_size){}

    template<bool POLICY>
    MessageResult push(Message&& message) {
        if (is_closed()) {
            return MessageResult::Closed;
        }
        {
            std::unique_lock lk{ mtx };
            if (m_queue.size() == max_queue_size) {
                if (POLICY == false) {
                    return MessageResult::Full;
                }
                else {
                    m_push_cvr.wait(lk, [this] {
                        return is_closed() || !(m_queue.size() < max_queue_size);
                        });
                    if (is_closed())
                        return MessageResult::Closed;
                }
            }
            m_queue.emplace_back(message);
        }
        m_pop_cvr.notify_one();
        return MessageResult::Ok;
    }

    template<bool POLICY>
    std::pair(Message,MessageResult) pop() {
        if (is_closed())
            return { {},MessageResult::Closed };
        Message mes;
        {
            std::unique_lock lk{ mtx };
            if (m_queue.size() == max_queue_size) {
                if (POLICY == false)
                    return { {},MessageResult::Empty };
                else {
                    m_pop_cvr.wait(lk, [this] { return IsClosed() || !m_queue.empty(); });
                    if (is_closed())
                    {
                        return { {},MessageResult::Closed };
                    }
                }
            }
            msg = std::move(m_queue.front());
            m_queue.pop_front();
        }
        m_push_cvr.notify_one();
        return { msg,MessageResult::Ok };

    }

    MessageResult close() {
        m_pop_cvr.notify_all();
        m_push_cvr.notify_all();
        m_state_closed.store(true, std::memory_order_release);
        return MessageResult::Ok;
    }
private:
    bool is_closed() {
        return(m_state_closed.load(std::memory_order_acquire) == true)
    }
    //block thread while queue not empty
    std::condition_variable m_pop_cvr;
    //block thread while queue not full
    std::condition_variable m_push_cvr;
    std::vector m_queue;
    std::size_t max_queue_size;
    std::mutex mtx;
    // state is atomic to avoid mutex lock while state checking
    std::atomic<bool> m_state_closed{false};
};

#endif THREAD_MESSAGE_H_