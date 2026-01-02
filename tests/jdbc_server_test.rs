use std::sync::Arc;
use tokio::sync::Mutex;
use remdb_server::jdbc_server::JdbcServer;
use sha2::{Sha256, Digest};
use hex;

#[test]
fn test_password_hash_generation() {
    // 测试密码哈希生成是否正确
    let password = "password123";
    let mut hasher = Sha256::new();
    hasher.update(password);
    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);
    
    // 验证哈希值是否符合预期
    assert_eq!(hash_str, "ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f");
}

#[test]
fn test_verify_credentials() {
    // 测试用户名密码验证
    let expected_username = "admin";
    let expected_password = "password123";
    
    // 生成密码哈希
    let mut hasher = Sha256::new();
    hasher.update(expected_password);
    let expected_hash = hex::encode(hasher.finalize());
    
    // 测试正确凭据
    assert!(JdbcServer::verify_credentials(expected_username, expected_password, expected_username, &expected_hash));
    
    // 测试错误用户名
    assert!(!JdbcServer::verify_credentials("wrong_user", expected_password, expected_username, &expected_hash));
    
    // 测试错误密码
    assert!(!JdbcServer::verify_credentials(expected_username, "wrong_password", expected_username, &expected_hash));
    
    // 测试空用户名
    assert!(!JdbcServer::verify_credentials("", expected_password, expected_username, &expected_hash));
    
    // 测试空密码
    assert!(!JdbcServer::verify_credentials(expected_username, "", expected_username, &expected_hash));
}

// 简化测试，只测试verify_credentials方法，避免访问私有字段
// 认证流程和SQL执行的测试可以在集成测试中完成
