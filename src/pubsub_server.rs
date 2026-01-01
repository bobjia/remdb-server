use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::udp_transport::{UdpTransport, UdpTransportConfig};

// PubSub服务器配置
pub struct PubSubServerConfig {
    pub enabled: bool,
    pub udp_config: UdpTransportConfig,
}

// PubSub服务器状态
pub enum PubSubServerState {
    Running,
    Stopped,
}

// PubSub服务器
pub struct PubSubServer {
    state: Arc<Mutex<PubSubServerState>>,
    udp_transport: Option<UdpTransport>,
    config: PubSubServerConfig,
}

impl Default for PubSubServerConfig {
    fn default() -> Self {
        PubSubServerConfig {
            enabled: false,
            udp_config: UdpTransportConfig::default(),
        }
    }
}

impl PubSubServer {
    // 创建新的PubSub服务器实例
    pub fn new(config: PubSubServerConfig) -> Self {
        PubSubServer {
            state: Arc::new(Mutex::new(PubSubServerState::Stopped)),
            udp_transport: None,
            config,
        }
    }
    
    // 启动PubSub服务器
    pub fn start(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            PubSubServerState::Running => {
                return Ok(());
            },
            PubSubServerState::Stopped => {
                if !self.config.enabled {
                    return Ok(());
                }
                
                // 创建UDP传输层
                let udp_transport = match UdpTransport::new(self.config.udp_config.clone()) {
                    Ok(transport) => transport,
                    Err(err) => {
                        return Err(format!("Failed to create UDP transport: {:?}", err));
                    },
                };
                
                // 启动UDP传输层
                let mut transport = udp_transport;
                transport.start();
                
                // 更新状态
                *state = PubSubServerState::Running;
                self.udp_transport = Some(transport);
                
                println!("PubSub server started on {}", self.config.udp_config.bind_address);
                Ok(())
            },
        }
    }
    
    // 停止PubSub服务器
    pub fn stop(&mut self) {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            PubSubServerState::Running => {
                // 停止UDP传输层
                if let Some(mut transport) = self.udp_transport.take() {
                    transport.stop();
                }
                
                // 更新状态
                *state = PubSubServerState::Stopped;
                println!("PubSub server stopped");
            },
            PubSubServerState::Stopped => {
                // 已经停止，无需操作
            },
        }
    }
    
    // 检查服务器状态
    pub fn is_running(&self) -> bool {
        let state = self.state.lock().unwrap();
        matches!(*state, PubSubServerState::Running)
    }
    
    // 获取当前配置
    pub fn get_config(&self) -> &PubSubServerConfig {
        &self.config
    }
    
    // 更新配置（需要重启服务器才能生效）
    pub fn update_config(&mut self, new_config: PubSubServerConfig) {
        self.config = new_config;
    }
}

// 测试PubSub服务器
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pubsub_server_start_stop() {
        let mut config = PubSubServerConfig::default();
        config.enabled = true;
        config.udp_config.bind_address = "127.0.0.1:0".to_string();
        
        let mut server = PubSubServer::new(config);
        
        // 启动服务器
        assert!(server.start().is_ok());
        assert!(server.is_running());
        
        // 停止服务器
        server.stop();
        assert!(!server.is_running());
    }
    
    #[test]
    fn test_pubsub_server_disabled() {
        let mut config = PubSubServerConfig::default();
        config.enabled = false;
        
        let mut server = PubSubServer::new(config);
        
        // 启动服务器（应该不实际启动）
        assert!(server.start().is_ok());
        assert!(!server.is_running());
    }
}