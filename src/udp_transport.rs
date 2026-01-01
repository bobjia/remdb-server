use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, Duration};
use std::thread;

// 消息类型枚举
#[derive(Debug, Clone, Copy)]
enum MessageType {
    Data = 0,
    ACK = 1,
    Heartbeat = 2,
    Subscribe = 3,
    Unsubscribe = 4,
    Publish = 5,
}

// UDP消息头
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct UdpHeader {
    msg_type: u8,
    seq: u32,
    topic_len: u8,
    data_len: u16,
}

// 消息结构体
struct UdpMessage {
    header: UdpHeader,
    topic: String,
    data: Vec<u8>,
}

// 等待ACK的消息
struct PendingMessage {
    message: Vec<u8>,
    addr: SocketAddr,
    sent_at: Instant,
    retry_count: u32,
}

// UDP传输配置
#[derive(Clone)]
pub struct UdpTransportConfig {
    pub bind_address: String,
    pub heartbeat_interval: Duration,
    pub retransmission_timeout: Duration,
    pub max_retransmissions: u32,
}

// UDP传输层
pub struct UdpTransport {
    socket: Arc<UdpSocket>,
    config: UdpTransportConfig,
    pending_messages: Arc<Mutex<HashMap<u32, PendingMessage>>>,
    next_seq: Arc<Mutex<u32>>,
    subscribers: Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    running: Arc<AtomicBool>,
    receive_thread: Option<thread::JoinHandle<()>>,
    retransmit_thread: Option<thread::JoinHandle<()>>,
    heartbeat_thread: Option<thread::JoinHandle<()>>,
}

impl Default for UdpTransportConfig {
    fn default() -> Self {
        UdpTransportConfig {
            bind_address: "0.0.0.0:8080".to_string(),
            heartbeat_interval: Duration::from_millis(1000),
            retransmission_timeout: Duration::from_millis(500),
            max_retransmissions: 3,
        }
    }
}

impl UdpTransport {
    // 创建新的UDP传输层实例
    pub fn new(config: UdpTransportConfig) -> std::io::Result<Self> {
        let socket = Arc::new(UdpSocket::bind(&config.bind_address)?);
        socket.set_nonblocking(true)?;
        
        Ok(UdpTransport {
            socket,
            config,
            pending_messages: Arc::new(Mutex::new(HashMap::new())),
            next_seq: Arc::new(Mutex::new(0)),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
            receive_thread: None,
            retransmit_thread: None,
            heartbeat_thread: None,
        })
    }
    
    // 启动传输层
    pub fn start(&mut self) {
        self.running.store(true, Ordering::Relaxed);
        
        // 启动接收线程
        let socket = self.socket.clone();
        let pending_messages = self.pending_messages.clone();
        let subscribers = self.subscribers.clone();
        let running = self.running.clone();
        let receive_thread = thread::spawn(move || {
            Self::receive_loop(socket, pending_messages, subscribers, running);
        });
        self.receive_thread = Some(receive_thread);
        
        // 启动重传线程
        let socket = self.socket.clone();
        let pending_messages = self.pending_messages.clone();
        let retransmission_timeout = self.config.retransmission_timeout;
        let max_retransmissions = self.config.max_retransmissions;
        let running = self.running.clone();
        let retransmit_thread = thread::spawn(move || {
            Self::retransmit_loop(socket, pending_messages, retransmission_timeout, max_retransmissions, running);
        });
        self.retransmit_thread = Some(retransmit_thread);
        
        // 启动心跳线程
        let socket = self.socket.clone();
        let subscribers = self.subscribers.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        let running = self.running.clone();
        let heartbeat_thread = thread::spawn(move || {
            Self::heartbeat_loop(socket, subscribers, heartbeat_interval, running);
        });
        self.heartbeat_thread = Some(heartbeat_thread);
    }
    
    // 停止传输层
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(thread) = self.receive_thread.take() {
            thread.join().ok();
        }
        if let Some(thread) = self.retransmit_thread.take() {
            thread.join().ok();
        }
        if let Some(thread) = self.heartbeat_thread.take() {
            thread.join().ok();
        }
    }
    
    // 接收消息循环
    fn receive_loop(
        socket: Arc<UdpSocket>,
        pending_messages: Arc<Mutex<HashMap<u32, PendingMessage>>>,
        subscribers: Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
        running: Arc<AtomicBool>,
    ) {
        let mut buf = [0u8; 65535];
        
        while running.load(Ordering::Relaxed) {
            match socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    if size < std::mem::size_of::<UdpHeader>() {
                        continue;
                    }
                    
                    // 解析消息头
                    let header = unsafe {
                        std::ptr::read_unaligned(buf.as_ptr() as *const UdpHeader)
                    };
                    
                    match header.msg_type {
                        // 数据消息
                        msg_type if msg_type == MessageType::Data as u8 => {
                            // 发送ACK
                            let ack_header = UdpHeader {
                                msg_type: MessageType::ACK as u8,
                                seq: header.seq,
                                topic_len: 0,
                                data_len: 0,
                            };
                            let ack_buf = unsafe {
                                std::slice::from_raw_parts(
                                    &ack_header as *const UdpHeader as *const u8,
                                    std::mem::size_of::<UdpHeader>(),
                                )
                            };
                            socket.send_to(ack_buf, addr).ok();
                            
                            // 处理数据消息
                            let topic_len = header.topic_len as usize;
                            let data_len = header.data_len as usize;
                            let total_len = std::mem::size_of::<UdpHeader>() + topic_len + data_len;
                            
                            if size < total_len {
                                continue;
                            }
                            
                            let topic = String::from_utf8_lossy(
                                &buf[std::mem::size_of::<UdpHeader>()..std::mem::size_of::<UdpHeader>() + topic_len],
                            ).to_string();
                            let _data = buf[std::mem::size_of::<UdpHeader>() + topic_len..total_len].to_vec();
                            
                            // 发布消息给订阅者
                            let subscribers_map = subscribers.read().unwrap();
                            if let Some(client_addrs) = subscribers_map.get(&topic) {
                                for client_addr in client_addrs {
                                    if *client_addr != addr {
                                        // 转发消息给订阅者
                                        let msg_buf = &buf[..total_len];
                                        socket.send_to(msg_buf, client_addr).ok();
                                    }
                                }
                            }
                        },
                        
                        // ACK消息
                        msg_type if msg_type == MessageType::ACK as u8 => {
                            // 从pending列表中移除
                            let mut pending = pending_messages.lock().unwrap();
                            pending.remove(&header.seq);
                        },
                        
                        // 心跳消息
                        msg_type if msg_type == MessageType::Heartbeat as u8 => {
                            // 回复心跳ACK
                            let ack_header = UdpHeader {
                                msg_type: MessageType::ACK as u8,
                                seq: header.seq,
                                topic_len: 0,
                                data_len: 0,
                            };
                            let ack_buf = unsafe {
                                std::slice::from_raw_parts(
                                    &ack_header as *const UdpHeader as *const u8,
                                    std::mem::size_of::<UdpHeader>(),
                                )
                            };
                            socket.send_to(ack_buf, addr).ok();
                        },
                        
                        // 订阅消息
                        msg_type if msg_type == MessageType::Subscribe as u8 => {
                            let topic_len = header.topic_len as usize;
                            let total_len = std::mem::size_of::<UdpHeader>() + topic_len;
                            
                            if size < total_len {
                                continue;
                            }
                            
                            let topic = String::from_utf8_lossy(
                                &buf[std::mem::size_of::<UdpHeader>()..total_len],
                            ).to_string();
                            
                            let mut subscribers_map = subscribers.write().unwrap();
                            subscribers_map.entry(topic.clone()).or_default().push(addr);
                            
                            // 发送ACK
                            let ack_header = UdpHeader {
                                msg_type: MessageType::ACK as u8,
                                seq: header.seq,
                                topic_len: 0,
                                data_len: 0,
                            };
                            let ack_buf = unsafe {
                                std::slice::from_raw_parts(
                                    &ack_header as *const UdpHeader as *const u8,
                                    std::mem::size_of::<UdpHeader>(),
                                )
                            };
                            socket.send_to(ack_buf, addr).ok();
                            
                            println!("Client {:?} subscribed to topic: {}", addr, topic);
                        },
                        
                        // 取消订阅消息
                        msg_type if msg_type == MessageType::Unsubscribe as u8 => {
                            let topic_len = header.topic_len as usize;
                            let total_len = std::mem::size_of::<UdpHeader>() + topic_len;
                            
                            if size < total_len {
                                continue;
                            }
                            
                            let topic = String::from_utf8_lossy(
                                &buf[std::mem::size_of::<UdpHeader>()..total_len],
                            ).to_string();
                            
                            let mut subscribers_map = subscribers.write().unwrap();
                            if let Some(client_addrs) = subscribers_map.get_mut(&topic) {
                                client_addrs.retain(|&a| a != addr);
                                if client_addrs.is_empty() {
                                    subscribers_map.remove(&topic);
                                }
                            }
                            
                            // 发送ACK
                            let ack_header = UdpHeader {
                                msg_type: MessageType::ACK as u8,
                                seq: header.seq,
                                topic_len: 0,
                                data_len: 0,
                            };
                            let ack_buf = unsafe {
                                std::slice::from_raw_parts(
                                    &ack_header as *const UdpHeader as *const u8,
                                    std::mem::size_of::<UdpHeader>(),
                                )
                            };
                            socket.send_to(ack_buf, addr).ok();
                            
                            println!("Client {:?} unsubscribed from topic: {}", addr, topic);
                        },
                        
                        // 发布消息
                        msg_type if msg_type == MessageType::Publish as u8 => {
                            let topic_len = header.topic_len as usize;
                            let data_len = header.data_len as usize;
                            let total_len = std::mem::size_of::<UdpHeader>() + topic_len + data_len;
                            
                            if size < total_len {
                                continue;
                            }
                            
                            let topic = String::from_utf8_lossy(
                                &buf[std::mem::size_of::<UdpHeader>()..std::mem::size_of::<UdpHeader>() + topic_len],
                            ).to_string();
                            
                            // 发送ACK
                            let ack_header = UdpHeader {
                                msg_type: MessageType::ACK as u8,
                                seq: header.seq,
                                topic_len: 0,
                                data_len: 0,
                            };
                            let ack_buf = unsafe {
                                std::slice::from_raw_parts(
                                    &ack_header as *const UdpHeader as *const u8,
                                    std::mem::size_of::<UdpHeader>(),
                                )
                            };
                            socket.send_to(ack_buf, addr).ok();
                            
                            // 转发消息给订阅者
                            let subscribers_map = subscribers.read().unwrap();
                            if let Some(client_addrs) = subscribers_map.get(&topic) {
                                for client_addr in client_addrs {
                                    if *client_addr != addr {
                                        let msg_buf = &buf[..total_len];
                                        socket.send_to(msg_buf, client_addr).ok();
                                    }
                                }
                            }
                            
                            println!("Published message to topic: {}, data len: {}", topic, data_len);
                        },
                        
                        _ => {}
                    }
                },
                Err(_) => {
                    // 非阻塞读取，忽略错误
                }
            }
            
            // 短暂休眠，避免CPU占用过高
            thread::sleep(Duration::from_millis(1));
        }
    }
    
    // 重传循环
    fn retransmit_loop(
        socket: Arc<UdpSocket>,
        pending_messages: Arc<Mutex<HashMap<u32, PendingMessage>>>,
        retransmission_timeout: Duration,
        max_retransmissions: u32,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::Relaxed) {
            let mut pending = pending_messages.lock().unwrap();
            let now = Instant::now();
            
            // 检查所有pending消息
            let mut to_remove = Vec::new();
            let mut to_update = Vec::new();
            
            // 第一次遍历：收集需要重传或移除的消息
            for (seq, msg) in pending.iter() {
                if now.duration_since(msg.sent_at) > retransmission_timeout {
                    if msg.retry_count >= max_retransmissions {
                        // 超过最大重传次数，移除
                        to_remove.push(*seq);
                    } else {
                        // 重传消息
                        socket.send_to(&msg.message, msg.addr).ok();
                        // 记录需要更新的消息
                        to_update.push((
                            *seq,
                            PendingMessage {
                                message: msg.message.clone(),
                                addr: msg.addr,
                                sent_at: now,
                                retry_count: msg.retry_count + 1,
                            }
                        ));
                    }
                }
            }
            
            // 第二次遍历：更新需要重传的消息
            for (seq, updated_msg) in to_update {
                pending.insert(seq, updated_msg);
            }
            
            // 移除超时且超过最大重传次数的消息
            for seq in to_remove {
                pending.remove(&seq);
            }
            
            drop(pending);
            
            // 休眠一段时间
            thread::sleep(Duration::from_millis(100));
        }
    }
    
    // 心跳循环
    fn heartbeat_loop(
        socket: Arc<UdpSocket>,
        subscribers: Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
        heartbeat_interval: Duration,
        running: Arc<AtomicBool>,
    ) {
        let heartbeat_msg = UdpHeader {
            msg_type: MessageType::Heartbeat as u8,
            seq: 0,
            topic_len: 0,
            data_len: 0,
        };
        let heartbeat_buf = unsafe {
            std::slice::from_raw_parts(
                &heartbeat_msg as *const UdpHeader as *const u8,
                std::mem::size_of::<UdpHeader>(),
            )
        };
        
        while running.load(Ordering::Relaxed) {
            // 发送心跳给所有订阅者
            let subscribers_map = subscribers.read().unwrap();
            let mut unique_addrs = Vec::new();
            
            // 收集所有唯一的订阅者地址
            for client_addrs in subscribers_map.values() {
                for addr in client_addrs {
                    if !unique_addrs.contains(addr) {
                        unique_addrs.push(*addr);
                    }
                }
            }
            
            // 发送心跳
            for addr in unique_addrs {
                socket.send_to(heartbeat_buf, addr).ok();
            }
            
            // 休眠心跳间隔
            thread::sleep(heartbeat_interval);
        }
    }
    
    // 发送消息
    pub fn send(&self, addr: SocketAddr, msg_type: MessageType, topic: &str, data: &[u8]) -> u32 {
        let mut next_seq = self.next_seq.lock().unwrap();
        let seq = *next_seq;
        *next_seq = seq.wrapping_add(1);
        drop(next_seq);
        
        // 构建消息头
        let header = UdpHeader {
            msg_type: msg_type as u8,
            seq,
            topic_len: topic.len() as u8,
            data_len: data.len() as u16,
        };
        
        // 构建消息
        let mut msg_buf = Vec::new();
        msg_buf.extend_from_slice(unsafe {
            std::slice::from_raw_parts(
                &header as *const UdpHeader as *const u8,
                std::mem::size_of::<UdpHeader>(),
            )
        });
        msg_buf.extend_from_slice(topic.as_bytes());
        msg_buf.extend_from_slice(data);
        
        // 发送消息
        self.socket.send_to(&msg_buf, addr).ok();
        
        // 添加到pending列表
        let mut pending = self.pending_messages.lock().unwrap();
        pending.insert(seq, PendingMessage {
            message: msg_buf,
            addr,
            sent_at: Instant::now(),
            retry_count: 0,
        });
        
        seq
    }
}

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;