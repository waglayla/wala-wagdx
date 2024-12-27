use crate::imports::AccountDescriptor;
use waglayla_wallet_core::prelude::AccountsSendResponse;

pub type Result<T> = std::result::Result<T, crate::error::Error>;
pub type UnlockResult = std::result::Result<Option<Vec<AccountDescriptor>>, waglayla_wallet_core::error::Error>;
pub type SendResult = std::result::Result<AccountsSendResponse, waglayla_wallet_core::error::Error>;
