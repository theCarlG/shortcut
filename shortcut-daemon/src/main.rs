use std::fs;
use std::os::unix::prelude::PermissionsExt;
use std::process::Stdio;

use shortcut_core::tokio::net::UnixListener;
use shortcut_core::tokio::process::Command;
use shortcut_core::tokio_stream::wrappers::UnixListenerStream;

use shortcut_core::tokio;
use shortcut_core::tonic::{self, transport::Server, Request, Response, Status};

use shortcut_core::wifi;
use shortcut_core::wifi::wifi_service_server;

use shortcut_core::ssh;
use shortcut_core::ssh::ssh_service_server;

use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Default)]
pub struct SshServer {}

#[tonic::async_trait]
impl ssh_service_server::SshService for SshServer {
    async fn set_enabled(
        &self,
        request: Request<ssh::SetEnabledRequest>,
    ) -> Result<Response<ssh::SetEnabledResponse>, Status> {
        let inner = request.into_inner();
        tracing::debug!("{:?}", inner);

        let cmd = Command::new("systemctl")
            .arg(if inner.enabled { "start" } else { "stop" })
            .arg("sshd")
            .status()
            .await
            .expect("failed to execute iw");

        let enabled = if cmd.success() {
            inner.enabled
        } else {
            !inner.enabled
        };

        let reply = ssh::SetEnabledResponse { enabled };
        Ok(Response::new(reply))
    }

    async fn get_enabled(
        &self,
        request: Request<ssh::GetEnabledRequest>,
    ) -> Result<Response<ssh::GetEnabledResponse>, Status> {
        let inner = request.into_inner();
        tracing::debug!("{:?}", inner);

        let mut sctl = Command::new("systemctl")
            .arg("status")
            .arg("sshd")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute systemctl");

        let sctl_stdout: Stdio = sctl
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("sctl did not give output");

        let mut grep = Command::new("grep")
            .arg("running")
            .stdin(sctl_stdout)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute grep");

        let grep_stdout: Stdio = grep
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("grep did not give output");

        let cmd = Command::new("wc")
            .arg("-l")
            .stdin(grep_stdout)
            .output()
            .await;

        let enabled = match cmd {
            Ok(output) => {
                let out = String::from_utf8(output.stdout).unwrap_or_else(|_| "0".to_string());

                match out.trim().parse::<i32>() {
                    Ok(val) => val == 1,
                    Err(err) => {
                        tracing::error!("error when parsing output from get_enabled: {err}");
                        false
                    }
                }
            }
            Err(err) => {
                tracing::error!("error when get_enabled: {err}");
                false
            }
        };

        let reply = ssh::GetEnabledResponse { enabled };

        Ok(Response::new(reply))
    }
}

#[derive(Debug, Default)]
pub struct WifiServer {}

#[tonic::async_trait]
impl wifi_service_server::WifiService for WifiServer {
    async fn set_power_save(
        &self,
        request: Request<wifi::SetPowerSaveRequest>,
    ) -> Result<Response<wifi::SetPowerSaveResponse>, Status> {
        let inner = request.into_inner();
        tracing::debug!("{:?}", inner);

        let cmd = Command::new("iw")
            .arg("dev")
            .arg(inner.device)
            .arg("set")
            .arg("power_save")
            .arg(if inner.enabled { "on" } else { "off" })
            .status()
            .await
            .expect("failed to execute iw");

        let enabled = if cmd.success() {
            inner.enabled
        } else {
            !inner.enabled
        };

        let reply = wifi::SetPowerSaveResponse { enabled };

        Ok(Response::new(reply))
    }

    async fn get_power_save(
        &self,
        request: Request<wifi::GetPowerSaveRequest>,
    ) -> Result<Response<wifi::GetPowerSaveResponse>, Status> {
        let inner = request.into_inner();
        tracing::debug!("{:?}", inner);

        let mut iw = Command::new("iw")
            .arg("dev")
            .arg(inner.device)
            .arg("get")
            .arg("power_save")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute iw");

        let iw_stdout: Stdio = iw
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("iw did not give output");

        let mut grep = Command::new("grep")
            .arg(" on")
            .stdin(iw_stdout)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute grep");

        let grep_stdout: Stdio = grep
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("grep did not give output");

        let cmd = Command::new("wc")
            .arg("-l")
            .stdin(grep_stdout)
            .output()
            .await;

        let enabled = match cmd {
            Ok(output) => {
                let out = String::from_utf8(output.stdout).unwrap_or_else(|_| "1".to_string());
                out.trim() == "1"
            }
            Err(err) => {
                tracing::error!("error when parsing output from get_power_save: {err}");
                false
            }
        };

        let reply = wifi::GetPowerSaveResponse { enabled };

        Ok(Response::new(reply))
    }

    async fn list_devices(
        &self,
        request: Request<wifi::ListDevicesRequest>,
    ) -> Result<Response<wifi::ListDevicesResponse>, Status> {
        let inner = request.into_inner();
        tracing::debug!("{:?}", inner);

        let mut iw = Command::new("iw")
            .arg("dev")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute iw");

        let iw_stdout: Stdio = iw
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("iw did not give output");

        let mut grep = Command::new("grep")
            .arg("Interface")
            .stdin(iw_stdout)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute grep");

        let grep_stdout: Stdio = grep
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("grep did not give output");

        let cmd = Command::new("awk")
            .arg("{ print $2 }")
            .stdin(grep_stdout)
            .output()
            .await;

        let devices: Vec<String> = match cmd {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(val) => val
                    .split('\n')
                    .filter_map(|s| {
                        if !s.is_empty() {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect(),
                Err(err) => {
                    tracing::error!("error when parsing output in list_devices: {err}");
                    vec![]
                }
            },
            Err(err) => {
                tracing::error!("error when list_devices: {err}");
                vec![]
            }
        };
        let reply = wifi::ListDevicesResponse { devices };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let collector = tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(EnvFilter::from_default_env());
    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
    tracing::info!("Logging initialized");

    let wifi_service = WifiServer::default();
    let ssh_service = SshServer::default();

    if std::path::Path::new(shortcut_core::SOCKET_PATH).exists() {
        tracing::warn!("Removing existing socket");
        fs::remove_file(shortcut_core::SOCKET_PATH).expect("unable to remove existing socket")
    }

    let uds = UnixListener::bind(shortcut_core::SOCKET_PATH)?;
    let uds_stream = UnixListenerStream::new(uds);

    tracing::info!("Listening on {}", shortcut_core::SOCKET_PATH);

    let mut perms = fs::metadata(shortcut_core::SOCKET_PATH)?.permissions();
    perms.set_mode(0o777);
    fs::set_permissions(shortcut_core::SOCKET_PATH, perms).unwrap();

    Server::builder()
        .add_service(wifi_service_server::WifiServiceServer::new(wifi_service))
        .add_service(ssh_service_server::SshServiceServer::new(ssh_service))
        .serve_with_incoming_shutdown(uds_stream, async move {
            tokio::signal::ctrl_c().await.unwrap();
            fs::remove_file(shortcut_core::SOCKET_PATH).expect("unable to remove existing socket")
        })
        .await?;

    Ok(())
}
