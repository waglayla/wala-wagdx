// use crate::events::Events;
use crate::imports::*;
use crate::interop::transport;
use crate::interop::transport::Target;
use crate::interop::{Adaptor, Request};
use borsh::BorshDeserialize;
pub use waglayla_wallet_core::api::transport::BorshCodec;
pub use waglayla_wallet_core::api::transport::{EventHandler, WalletServer};

pub struct Client {
    _sender: Arc<dyn transport::Sender>,
    // application_events: ApplicationEventsChannel,
    adaptor: Arc<Adaptor>,
}

impl Client {
    pub fn new(
        _sender: Arc<dyn transport::Sender>,
        // application_events: ApplicationEventsChannel,
    ) -> Self {
        // let adaptor = Arc::new(Adaptor::new(_sender.clone(), application_events.clone()));
        let adaptor = Arc::new(Adaptor::new(_sender.clone(), None));
        Self {
            _sender,
            // application_events,
            adaptor,
        }
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        self.adaptor.clone().init().await
    }

    pub fn adaptor(&self) -> &Arc<Adaptor> {
        &self.adaptor
    }

    pub async fn handle_message(&self, target: Target, data: Vec<u8>) -> Result<Option<Vec<u8>>> {
        match target {
            Target::Wallet => {
                let event = Box::new(waglayla_wallet_core::events::Events::try_from_slice(data.as_slice())?);

                // self.application_events
                //     .sender
                //     .try_send(waglayla_ng_core::events::Events::Wallet { event })
                //     .unwrap();

                Ok(None)
            }
            Target::Runtime => Ok(None),
            Target::Adaptor => {
                let action = Request::try_from_slice(&data)?;
                let response = self.adaptor.clone().handle_message(action).await?;
                Ok(Some(response))
            }
        }
    }
}
