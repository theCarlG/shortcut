pub use futures;
pub use tokio;
pub use tokio_stream;
pub use tonic;
pub use tower;

pub const SOCKET_PATH: &str = "/tmp/shortcutd.sock";

pub mod wifi {
    tonic::include_proto!("shortcut.wifi");
}

pub mod ssh {
    tonic::include_proto!("shortcut.ssh");
}
