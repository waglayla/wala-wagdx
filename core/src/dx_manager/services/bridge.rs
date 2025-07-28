include!(concat!(env!("OUT_DIR"), "/embedded_executable.rs"));

use crate::imports::*;
use std::fs::File;
use std::io::{Write};
use std::env;
use std::process::Stdio;
use tokio::sync::Mutex;
use tokio::process::{Command, Child};
use waglayla_wallet_core::storage::local::storage::Storage;
use tokio::io::{AsyncRead, AsyncBufReadExt, BufReader};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub enum BridgeEvents {
  Enable,
  Disable,
  Exit,
}

pub struct BridgeService {
  pub application_events: ApplicationEventsChannel,
  pub service_events: Channel<BridgeEvents>,
  pub task_ctl: Channel<()>,
  pub is_enabled: Arc<AtomicBool>,
  pub bridge_sender: Sender<DaemonMessage>,
}

impl BridgeService {
  pub fn new(application_events: ApplicationEventsChannel, settings: &Settings, bridge_sender: Sender<DaemonMessage>) -> Self {
    Self {
      application_events,
      service_events: Channel::unbounded(),
      task_ctl: Channel::oneshot(),
      is_enabled: Arc::new(AtomicBool::new(
        settings.node.node_kind == WagLayladNodeKind::IntegratedAsDaemon &&
        settings.node.enable_bridge
      )),
      bridge_sender,
    }
  }

  pub fn enable(&self) {
    self.service_events
      .sender
      .try_send(BridgeEvents::Enable)
      .unwrap();
  }

  pub fn exit(&self) {
    self.service_events
      .sender
      .try_send(BridgeEvents::Exit)
      .unwrap();
  }

  pub fn disable(&self) {
    self.service_events
      .sender
      .try_send(BridgeEvents::Disable)
      .unwrap();
  }

  async fn pipe_output<R: AsyncRead + Unpin + Send + 'static>(
    reader: R,
    sender: Sender<DaemonMessage>,
    source: &str,
  ) {
    let mut reader = BufReader::new(reader).lines();
    while let Ok(Some(line)) = reader.next_line().await {
      let log_message = format!("[{}] {}", source, line);

      if sender.send(DaemonMessage(log_message.clone())).await.is_err() {
        eprintln!("Failed to send {} output to bridge_sender", source);
        break;
      }
    }
  }

  pub fn update_services(&self, node_settings: &NodeSettings, options: Option<RpcOptions>) {
    match node_settings.node_kind {
      WagLayladNodeKind::IntegratedAsDaemon => {
        if (node_settings.enable_bridge) {
          self.enable();
        } else {
          self.disable();
        }
      },
      _ => {
        self.disable();
      }
    }
  }
}

#[async_trait]
impl Service for BridgeService {
  fn name(&self) -> &'static str {
    "wala-stratum-bridge"
  }

  async fn rpc_attach(self: Arc<Self>, _rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
    Ok(())
  }

  async fn rpc_detach(self: Arc<Self>) -> Result<()> {
    Ok(())
  }

  async fn launch(self: Arc<Self>) -> Result<()> {
    let this = self.clone();
    let _application_events_sender = self.application_events.sender.clone();

    #[cfg(windows)]
    let go_binary_name = "bridge.exe";
  
    #[cfg(not(windows))]
    let go_binary_name = "bridge";
  
    let bin = Storage::try_new(go_binary_name)?;
    let target_path = bin.filename();
  
    if !target_path.exists() {
      let mut temp_file = File::create(target_path)?;
      temp_file.write_all(BINARY)?;
  
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(target_path, std::fs::Permissions::from_mode(0o755))?;
      }
    }
  
    let mut backoff = 1;
    const MAX_BACKOFF: u64 = 16;

    let mut exit_requested = false;

    loop {
      let mut command = Command::new(&target_path);

      command
        .current_dir(target_path.parent().unwrap_or_else(|| Path::new(".")))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

      #[cfg(windows)]
      {
        command.creation_flags(CREATE_NO_WINDOW);
      }

      if self.is_enabled.load(Ordering::Relaxed) == true {
        let mut child_process = match
          command.spawn()
        {
          Ok(child) => child,
          Err(e) => {
            eprintln!("Failed to start bridge process: {}. Retrying...", e);
            tokio::time::sleep(Duration::from_secs(backoff)).await;
            backoff = (backoff * 2).min(MAX_BACKOFF);
            continue;
          }
        };

        if let Some(stdout) = child_process.stdout.take() {
          let sender = self.bridge_sender.clone();
          tokio::spawn(Self::pipe_output(stdout, sender, "stdout"));
        }
        
        if let Some(stderr) = child_process.stderr.take() {
          let sender = self.bridge_sender.clone();
          tokio::spawn(Self::pipe_output(stderr, sender, "stderr"));
        }

        loop {
          select! {
            msg = this.as_ref().service_events.receiver.recv().fuse() => {
              if let Ok(event) = msg {
                match event {
                  BridgeEvents::Enable => {
                    self.is_enabled.store(true, Ordering::Relaxed);
                  },
                  BridgeEvents::Disable => {
                    self.is_enabled.store(false, Ordering::Relaxed);
                    let _ = child_process.kill().await.expect("failed to kill bridge");
                    let _ = child_process.wait().await.expect("Failed to wait for bridge exit");
                    break;
                  },
                  BridgeEvents::Exit => {
                    let _ = child_process.kill().await.expect("failed to kill bridge");
                    let _ = child_process.wait().await.expect("Failed to wait for bridge exit");
                    exit_requested = true;
                    break;
                  }
                }
              } else {
                break;
              }
            }
            _ = tokio::time::sleep(Duration::from_secs(1)).fuse() => {
              if let Ok(Some(status)) = child_process.try_wait() {
                if !exit_requested {
                  eprintln!("Bridge process exited with status: {}. Restarting...", status);
                }
                break;
              }
            }
          }
        }

        tokio::time::sleep(Duration::from_secs(backoff)).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
      } else {
        loop {
          select! {
            msg = this.as_ref().service_events.receiver.recv().fuse() => {
              if let Ok(event) = msg {
                match event {
                  BridgeEvents::Enable => {
                    self.is_enabled.store(true, Ordering::Relaxed);
                    break;
                  },
                  BridgeEvents::Disable => {
                    self.is_enabled.store(false, Ordering::Relaxed);
                  },
                  BridgeEvents::Exit => {
                    exit_requested = true;
                    break;
                  }
                }
              } else {
                break;
              }
            }
            _ = tokio::time::sleep(Duration::from_secs(1)).fuse() => {}
          }
        }
      }

      if exit_requested {
        break;
      }
    }

    this.task_ctl.send(()).await?;
    Ok(())
  }

  fn terminate(self: Arc<Self>) {
    self.service_events
      .sender
      .try_send(BridgeEvents::Exit)
      .unwrap();
  }

  async fn join(self: Arc<Self>) -> Result<()> {
    self.task_ctl.recv().await.unwrap();
    Ok(())
  }
}