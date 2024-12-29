use crate::imports::*;
use waglayla_bip32::{Language, Mnemonic, WordCount};
use waglayla_wallet_core::{
  wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, 
  encryption::EncryptionKind, 
  api::{AccountsDiscoveryRequest, AccountsDiscoveryKind},
  tx::{GeneratorSummary, PaymentDestination, Fees},
  prelude::AccountsEstimateRequest
};
use egui::{ColorImage, TextureHandle};
use slug::slugify;

use waglayla_wallet_core::account::{
  BIP32_ACCOUNT_KIND, KEYPAIR_ACCOUNT_KIND, LEGACY_ACCOUNT_KIND, MULTISIG_ACCOUNT_KIND,
};

pub struct AccountContext {
  pub initial_address: Option<String>,
  pub initial_qr: Option<load::Bytes>,
  pub current_address: Option<String>,
  pub current_qr: Option<load::Bytes>,
  pub uri: Option<String>,
}

impl AccountContext {
  pub fn new(descriptor: &AccountDescriptor) -> Option<Arc<Self>> {
    let initial_address = if let Some(receive_address) = descriptor.receive_address() {
      Some(receive_address.to_string())
    } else {
      None
    };
    let initial_qr = generate_qr_code_svg(initial_address.clone().unwrap()).ok()?;

    Some(Arc::new(Self {
      initial_address: initial_address.clone(),
      initial_qr: Some(initial_qr.clone().as_bytes().to_vec().into()),
      current_address: initial_address,
      current_qr: Some(initial_qr.as_bytes().to_vec().into()),
      uri: None,
    }))
  }

  pub fn update_current_address(&mut self, new_address: String) -> Result<()> {
    self.current_address = Some(new_address.clone());
    self.current_qr = Some(generate_qr_code_svg(new_address)?.as_bytes().to_vec().into());
    Ok(())
  }

  pub fn update_payment_uri(&mut self, amount: u64, label: Option<String>) -> Result<()> {
    let address = self.current_address.as_ref()
      .ok_or_else(|| Error::custom("No current address available"))?;
    
    let mut uri = format!("waglayla:{}", address);
    
    let mut params = vec![];
    params.push(format!("amount={}", amount));
    
    if let Some(l) = label {
        params.push(format!("label={}", l));
    }
    
    if !params.is_empty() {
      uri.push('?');
      uri.push_str(&params.join("&"));
    }

    self.uri = Some(uri.clone());
    self.current_qr = Some(generate_qr_code_svg(uri)?.as_bytes().to_vec().into());
    
    Ok(())
  }

  // Reset to initial state
  pub fn reset(&mut self) {
    self.current_address = self.initial_address.clone();
    self.current_qr = self.initial_qr.clone();
    self.uri = None;
  }
}

struct Inner {
  id: AccountId,
  account_kind: AccountKind,
  balance: Mutex<Option<Balance>>,
  descriptor: Mutex<AccountDescriptor>,
  context: Mutex<Option<Arc<AccountContext>>>,
  transactions: Mutex<TransactionCollection>,
  max_spend: AtomicU64,
  total_transaction_count: AtomicU64,
  is_loading: AtomicBool,
  network: Mutex<Network>,
}

impl Inner {
  fn new(descriptor: AccountDescriptor) -> Self {
    let context = AccountContext::new(&descriptor);
    Self {
      id: *descriptor.account_id(),
      account_kind: *descriptor.account_kind(),
      balance: Mutex::new(None),
      descriptor: Mutex::new(descriptor),
      context: Mutex::new(context),
      transactions: Mutex::new(TransactionCollection::default()),
      max_spend: AtomicU64::new(0),
      total_transaction_count: AtomicU64::new(0),
      is_loading: AtomicBool::new(true),
      network: Mutex::new(Network::Mainnet),
    }
  }
}

#[derive(Clone)]
pub struct Account {
  inner: Arc<Inner>,
}

impl Account {
  pub fn from(descriptor: AccountDescriptor) -> Self {
    Self {
      inner: Arc::new(Inner::new(descriptor)),
    }
  }

  pub fn descriptor(&self) -> MutexGuard<'_, AccountDescriptor> {
    self.inner.descriptor.lock().unwrap()
  }

  pub fn derive_new_address(&self) -> Result<Option<Address>> {
    Ok(self.descriptor().receive_address().clone())
  }

  pub fn update_current_address(&self, new_address: String) -> Result<()> {
    let mut context_lock = self.inner.context.lock().unwrap();
    Arc::get_mut(context_lock
      .as_mut()
      .ok_or_else(|| Error::custom("Account context not initialized"))?
    )
      .ok_or_else(|| Error::custom("Unable to get mutable reference to context"))?
      .update_current_address(new_address)
    }

  pub fn update_payment_uri(&self, amount: u64, label: Option<String>) -> Result<()> {
    let mut context_lock = self.inner.context.lock().unwrap();
    Arc::get_mut(context_lock
      .as_mut()
      .ok_or_else(|| return Error::custom("Account context not initialized"))?
    )
      .ok_or_else(|| Error::custom("Unable to get mutable reference to context"))?
      .update_payment_uri(amount, label)
    }

  pub fn root_address(&self) -> Result<String> {
    let mut context_lock = self.inner.context.lock().unwrap();
    let address = Arc::get_mut(context_lock
      .as_mut()
      .ok_or_else(|| return Error::custom("Account context not initialized"))?
    )
      .ok_or_else(|| Error::custom("Unable to get mutable reference to context"))?
      .initial_address
      .clone();
    Ok(address.unwrap())
  }

  pub fn receive_address(&self) -> String {
    let address = self.derive_new_address().unwrap().unwrap().to_string();
    self.update_current_address(address.clone());
    address
  }

  pub fn reset_context(&self) -> Result<()> {
    let mut context_lock = self.inner.context.lock().unwrap();
    let result = Arc::get_mut(context_lock
      .as_mut()
      .ok_or_else(|| Error::custom("Account context not initialized"))?
    )
      .ok_or_else(|| Error::custom("Unable to get mutable reference to context"))?
      .reset();
    Ok(result)
  }

  // Method to get current QR code
  pub fn root_qr_code(&self) -> Option<load::Bytes> {
    self.inner.context.lock().unwrap()
      .as_ref()
      .map(|c| c.initial_qr.clone())
      .flatten()
  }

  pub fn current_qr_code(&self) -> Option<load::Bytes> {
    self.inner.context.lock().unwrap()
      .as_ref()
      .map(|c| c.current_qr.clone())
      .flatten()
  }

  // Example method to generate QR code for the receive address
  pub fn receive_address_qr_code(&self) -> Result<String> {
    let address = self.receive_address().to_string();
    generate_qr_code_svg(address)
  }

  
  pub fn transactions(&self) -> MutexGuard<'_, TransactionCollection> {
    self.inner.transactions.lock().unwrap()
  }

  pub fn transactions_mut(&self) -> MutexGuard<'_, TransactionCollection> {
    self.inner.transactions.lock().unwrap()
  }

  pub fn id(&self) -> AccountId {
    self.inner.id
  }

  pub fn name_or_id(&self) -> String {
    self.descriptor().name_or_id()
  }

  pub fn requires_bip39_passphrase(&self, core: &Core) -> bool {
    let descriptor = self.descriptor();
    let prv_key_data_ids = descriptor.prv_key_data_ids();
    core.prv_key_data_map()
      .as_ref()
      .map(|prv_key_data_map| {
        prv_key_data_ids.into_iter().any(|prv_key_data_id| {
          prv_key_data_map
            .get(&prv_key_data_id)
            .map(|prv_key_data_info| prv_key_data_info.requires_bip39_passphrase())
            .unwrap_or(false)
        })
      })
      .unwrap_or(false)
  }

  pub fn account_kind(&self) -> &AccountKind {
    &self.inner.account_kind
  }

  pub fn balance(&self) -> Option<Balance> {
    self.inner.balance.lock().unwrap().clone()
  }

  pub fn update_theme(&self) {
    let descriptor = self.descriptor().clone();
  }

  pub fn context(&self) -> Option<Arc<AccountContext>> {
    self.inner.context.lock().unwrap().clone()
  }

  pub fn update(&self, descriptor: AccountDescriptor) {
    println!("$$$$$ UPDATING ACCOUNT: {:?}", descriptor);

    *self.inner.descriptor.lock().unwrap() = descriptor;
  }

  pub fn update_balance(&self, balance: Option<Balance>) -> Result<()> {
    *self.inner.balance.lock().unwrap() = balance;
    Ok(())
  }

  pub fn update_network(&self, network: Network) {
    *self.inner.network.lock().unwrap() = network;
  }

  pub fn set_loading(&self, is_loading: bool) {
    self.inner.is_loading.store(is_loading, Ordering::SeqCst);
  }

  pub fn is_loading(&self) -> bool {
    self.inner.is_loading.load(Ordering::SeqCst)
  }

  pub fn set_transaction_count(&self, count: u64) {
    self.inner
      .total_transaction_count
      .store(count, Ordering::SeqCst);
  }

  pub fn transaction_count(&self) -> u64 {
    self.inner.total_transaction_count.load(Ordering::SeqCst)
  }

  pub fn load_transactions(
    &self,
    mut transactions: Vec<Arc<TransactionRecord>>,
    total: u64,
  ) -> Result<()> {
    self.transactions().clear();

    transactions.sort_by(|a, b| b.block_daa_score.cmp(&a.block_daa_score));

    self.set_transaction_count(total);
    self.transactions()
      .load(transactions.into_iter().map(|t| t.into()));

    Ok(())
  }

  pub fn clear_transactions(&self) {
    self.set_transaction_count(0);
    self.transactions().clear();
  }
}

impl IdT for Account {
  type Id = AccountId;

  fn id(&self) -> &Self::Id {
    &self.inner.id
  }
}

impl std::fmt::Debug for Account {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(&self.id(), f)
  }
}

pub type AccountGroup = Collection<AccountId, Account>;

pub trait DescribeAccount {
  fn describe(&self) -> (&'static str, &'static str);
}

impl DescribeAccount for AccountKind {
  fn describe(&self) -> (&'static str, &'static str) {
    match self.as_ref() {
      LEGACY_ACCOUNT_KIND => ("Legacy Account", "N/A"),
      BIP32_ACCOUNT_KIND => ("WagLayla Core BIP32", "waglayla-cli wallet"),
      MULTISIG_ACCOUNT_KIND => ("Multi-Signature", ""),
      KEYPAIR_ACCOUNT_KIND => ("Keypair", "secp256k1"),
      _ => ("", ""),
    }
  }
}