# ThreadQueue
test multithread  queque
asynchronous message exchange queue between ‘Writer’ (aka ‘Producer’) 
thread and “Reader” (aka ‘Consumer’) thread. This queue is supposed to be used for temporary traffic peaks 
mitigation  (e.g. 'Reader’ is consuming messages with a constant rate, while ‘Writer’ produces messages 
randomly)  
