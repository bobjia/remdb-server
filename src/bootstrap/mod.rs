pub mod platform;
pub mod service;

pub use platform::{WindowsPlatform, init_platform};
pub use service::{DatabaseContext, ServiceStarter};
