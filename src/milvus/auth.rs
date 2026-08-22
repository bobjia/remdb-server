use sha2::{Digest, Sha256};
use warp::Filter;

use crate::milvus::error::MilvusError;

pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn auth_filter(
    expected_hash: String,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and_then(move |auth_header: Option<String>| {
            let expected = expected_hash.clone();
            async move {
                match auth_header {
                    Some(header) => {
                        let token = if header.starts_with("Bearer ") {
                            &header[7..]
                        } else {
                            return Err(warp::reject::custom(MilvusError::AuthenticationFailed));
                        };

                        let provided_hash = hash_api_key(token);
                        if provided_hash == expected {
                            Ok(())
                        } else {
                            Err(warp::reject::custom(MilvusError::AuthenticationFailed))
                        }
                    }
                    None => {
                        Err(warp::reject::custom(MilvusError::AuthenticationFailed))
                    }
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_api_key() {
        let hash = hash_api_key("test-key");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_consistency() {
        let h1 = hash_api_key("test-key");
        let h2 = hash_api_key("test-key");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_different_keys_different_hashes() {
        let h1 = hash_api_key("key1");
        let h2 = hash_api_key("key2");
        assert_ne!(h1, h2);
    }
}
