use crate::imports::*;
use waglayla_bip32::{Language, Mnemonic, WordCount};
use waglayla_wallet_core::{wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, encryption::EncryptionKind, api::{AccountsDiscoveryRequest, AccountsDiscoveryKind}};
use slug::slugify;

pub async fn create_wallet(
  wallet_name: String,
  account_name: Option<String>,
  wallet_secret_str: String,
  payment_secret_str: Option<String>,
  word_count: WordCount,
) -> Result<()> {
  let manager = manager().clone();
  let wallet = manager.wallet();

  let file_name = slugify(wallet_name.clone());

  let wallet_secret = Secret::from(wallet_secret_str);
  let payment_secret = payment_secret_str.map(|s| Secret::from(s));

  let account_name = account_name.unwrap_or_else(|| "Account 1".to_string());

  wallet.clone().batch().await?;

  let wallet_args = WalletCreateArgs::new(
    Some(wallet_name.clone()),
    Some(file_name),
    EncryptionKind::XChaCha20Poly1305,
    None,
    false
  );
  
  wallet.clone().wallet_create(wallet_secret.clone(), wallet_args).await?;

  let mnemonic = Mnemonic::random(word_count, Language::default())?;
  let mnemonic_phrase_string = mnemonic.phrase_string();
  let prv_key_data_args = PrvKeyDataCreateArgs::new(
    None,
    payment_secret.clone(),
    Secret::from(mnemonic_phrase_string.clone()),
  );

  let prv_key_data_id = wallet.clone().prv_key_data_create(wallet_secret.clone(), prv_key_data_args).await?;

  let account_create_args = AccountCreateArgs::new_bip32(
      prv_key_data_id,
      payment_secret.clone(),
      Some(account_name),
      None,
  );

  let account_descriptor = wallet.clone().accounts_create(wallet_secret.clone(), account_create_args).await?;

  wallet.clone().flush(wallet_secret).await?;
  Ok(())
}