use crate::imports::*;

pub mod waglayla;
pub use waglayla::WaglaylaService;

pub mod peers;
pub use peers::PeerMonitorService;

pub mod stats;
pub use stats::StatMonitorService;

/// Service is a core component of the Waglayla NG application responsible for
/// running application services and communication between these services.
#[async_trait]
pub trait Service: Sync + Send {
  fn name(&self) -> &'static str;

  async fn launch(self: Arc<Self>) -> Result<()>;
  fn terminate(self: Arc<Self>);
  async fn join(self: Arc<Self>) -> Result<()>;

  async fn rpc_attach(self: Arc<Self>, _rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
    Ok(())
  }

  async fn rpc_detach(self: Arc<Self>) -> Result<()> {
    Ok(())
  }

  async fn rpc_connect(self: Arc<Self>) -> Result<()> {
    Ok(())
  }

  async fn rpc_disconnect(self: Arc<Self>) -> Result<()> {
    Ok(())
  }
}
