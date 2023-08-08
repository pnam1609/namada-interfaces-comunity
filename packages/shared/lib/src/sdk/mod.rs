use std::str::FromStr;

use crate::utils::to_js_result;
use crate::{
    rpc_client::HttpClient,
    sdk::masp::WebShieldedUtils,
    utils::{set_panic_hook, to_bytes},
};
use borsh::{BorshDeserialize, BorshSerialize};
use namada::ledger::signing::SigningTxData;
use namada::ledger::tx::Error;
use namada::types::address::Address;
use namada::types::key::common::SecretKey;
use namada::types::tx::TxBuilder;
use namada::{
    ledger::{
        args,
        masp::ShieldedContext,
        signing,
        wallet::{Store, Wallet},
    },
    proto::{Section, Tx},
    types::key::common::PublicKey,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

pub mod masp;
mod signature;
mod tx;
mod wallet;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum TxType {
    Bond = 1,
    Unbond = 2,
    Withdraw = 3,
    Transfer = 4,
    IBCTransfer = 5,
    RevealPK = 6,
}

/// Represents the Sdk public API.
#[wasm_bindgen]
pub struct Sdk {
    client: HttpClient,
    wallet: Wallet<wallet::BrowserWalletUtils>,
    shielded_ctx: ShieldedContext<masp::WebShieldedUtils>,
}

#[wasm_bindgen]
/// Sdk mostly wraps the logic of the Sdk struct members, making it a part of public API.
/// For more details, navigate to the corresponding modules.
impl Sdk {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String) -> Self {
        set_panic_hook();
        Sdk {
            client: HttpClient::new(url),
            wallet: Wallet::new(wallet::STORAGE_PATH.to_owned(), Store::default()),
            shielded_ctx: ShieldedContext::default(),
        }
    }

    pub async fn has_masp_params() -> Result<JsValue, JsValue> {
        let has = has_masp_params().await?;

        Ok(js_sys::Boolean::from(has.as_bool().unwrap()).into())
    }

    pub async fn fetch_and_store_masp_params() -> Result<(), JsValue> {
        fetch_and_store_masp_params().await?;
        Ok(())
    }

    pub async fn load_masp_params(&mut self) -> Result<(), JsValue> {
        let params = get_masp_params().await?;
        let params_iter = js_sys::try_iter(&params)?.ok_or_else(|| "Can't iterate over JsValue")?;
        let mut params_bytes = params_iter.map(|p| to_bytes(p.unwrap()));

        let spend = params_bytes.next().unwrap();
        let output = params_bytes.next().unwrap();
        let convert = params_bytes.next().unwrap();

        // We are making sure that there are no more params left
        assert_eq!(params_bytes.next(), None);

        self.shielded_ctx = WebShieldedUtils::new(spend, output, convert);

        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        wallet::encode(&self.wallet)
    }

    pub fn decode(&mut self, data: Vec<u8>) -> Result<(), JsError> {
        let wallet = wallet::decode(data)?;
        self.wallet = wallet;
        Ok(())
    }

    pub fn clear_storage(&mut self) -> Result<(), JsError> {
        self.wallet = Wallet::new(wallet::STORAGE_PATH.to_owned(), Store::default());
        Ok(())
    }

    pub fn add_key(&mut self, private_key: &str, password: Option<String>, alias: Option<String>) {
        wallet::add_key(&mut self.wallet, private_key, password, alias)
    }

    pub fn add_spending_key(&mut self, xsk: &str, password: Option<String>, alias: &str) {
        wallet::add_spending_key(&mut self.wallet, xsk, password, alias)
    }

    async fn submit_reveal_pk(
        &mut self,
        args: &args::Tx,
        pk: &PublicKey,
        gas_payer: Option<SecretKey>,
    ) -> Result<(), JsError> {
        // Build a transaction to reveal the signer of this transaction
        let mut tx_builder = namada::ledger::tx::build_reveal_pk(
            &self.client,
            args,
            //TODO: This is only needed for logging, I imagine it will be cleaned up in Namada
            &args.gas_token,
            &pk,
            // In the case of web interface gas_payer is the same as the signer
            &pk,
        )
        .await?;

        // Add gas payer - hardware wallets should do it automatically
        if let Some(gas_payer) = gas_payer {
            tx_builder = tx_builder.add_gas_payer(gas_payer);
        }

        namada::ledger::tx::process_tx(&self.client, &mut self.wallet, &args, tx_builder.build())
            .await?;

        Ok(())
    }

    /// Sign and submit transactions
    async fn sign_and_process_tx(
        &mut self,
        args: args::Tx,
        tx_builder: TxBuilder,
        signing_data: SigningTxData,
    ) -> Result<(), JsError> {
        // We are revealing the signer of this transaction(if needed)
        // We only support one signer(for now)
        let pk = &signing_data
            .public_keys
            .clone()
            .into_iter()
            .nth(0)
            .expect("No public key provided");
        let sk = self
            .wallet
            .find_key_by_pk(pk, args.clone().password)
            .expect("No secret key found");

        // Submit a reveal pk tx if necessary
        // TODO: do not submit when faucet
        self.submit_reveal_pk(&args, &pk, Some(sk)).await?;

        // Sign tx
        let tx_builder = signing::sign_tx(&mut self.wallet, &args, tx_builder, signing_data)?;
        let tx = tx_builder.build();
        // Submit tx
        namada::ledger::tx::process_tx(&self.client, &mut self.wallet, &args, tx).await?;

        Ok(())
    }

    /// Submit signed reveal pk tx
    pub async fn submit_signed_reveal_pk(
        &mut self,
        tx_msg: &[u8],
        tx_bytes: &[u8],
        raw_sig_bytes: &[u8],
        wrapper_sig_bytes: &[u8],
    ) -> Result<(), JsError> {
        let reveal_pk_tx = self.sign_tx(tx_bytes, raw_sig_bytes, wrapper_sig_bytes)?;
        let args = tx::tx_args_from_slice(&tx_msg)?;

        namada::ledger::tx::process_tx(&self.client, &mut self.wallet, &args, reveal_pk_tx).await?;

        Ok(())
    }

    /// Build transaction for specified type, return bytes to client
    pub async fn build_tx(
        &mut self,
        tx_type: TxType,
        tx_msg: &[u8],
        gas_payer: String,
    ) -> Result<JsValue, JsError> {
        //TODO: verify if this works
        let gas_payer = PublicKey::from_str(&gas_payer)?;

        let tx_builder = match tx_type {
            TxType::Bond => {
                let args = tx::bond_tx_args(tx_msg, None)?;
                let bond = namada::ledger::tx::build_bond(&self.client, args.clone(), &gas_payer)
                    .await
                    .map_err(JsError::from)?;
                bond
            }
            TxType::RevealPK => {
                let args = tx::tx_args_from_slice(tx_msg)?;

                let public_key = match args.verification_key.clone() {
                    Some(v) => PublicKey::from(v),
                    _ => {
                        return Err(JsError::new(
                            "verification_key is required in this context!",
                        ))
                    }
                };

                let address = Address::from(&public_key);

                let reveal_pk = namada::ledger::tx::build_reveal_pk(
                    &self.client,
                    &args.clone(),
                    &address,
                    &public_key,
                    &gas_payer,
                )
                .await?;

                reveal_pk
            }
            TxType::Transfer => {
                let args = tx::transfer_tx_args(tx_msg, None, None)?;
                let (tx_builder, _) = namada::ledger::tx::build_transfer(
                    &self.client,
                    &mut self.shielded_ctx,
                    args.clone(),
                    &gas_payer,
                )
                .await?;
                tx_builder
            }
            TxType::IBCTransfer => {
                let args = tx::ibc_transfer_tx_args(tx_msg, None)?;
                let ibc_transfer = namada::ledger::tx::build_ibc_transfer(
                    &self.client,
                    args.clone(),
                    &gas_payer
                )
                .await?;
                ibc_transfer
            }
            TxType::Unbond => {
                let args = tx::unbond_tx_args(tx_msg, None)?;
                let (tx_builder, _) = namada::ledger::tx::build_unbond(
                    &self.client,
                    &mut self.wallet,
                    args.clone(),
                    &gas_payer,
                )
                .await?;
                tx_builder
            }
            TxType::Withdraw => {
                let args = tx::withdraw_tx_args(tx_msg, None)?;
                namada::ledger::tx::build_withdraw(&self.client, args.clone(), &gas_payer).await?
            }
        };
        let tx = tx_builder.build();

        to_js_result(tx.try_to_vec()?)
    }

    // Append signatures and return tx bytes
    fn sign_tx(
        &self,
        tx_bytes: &[u8],
        raw_sig_bytes: &[u8],
        wrapper_sig_bytes: &[u8],
    ) -> Result<Tx, JsError> {
        let mut tx: Tx = Tx::try_from_slice(tx_bytes)?;

        let raw_sig = signature::construct_signature(raw_sig_bytes, &tx)?;
        tx.add_section(Section::Signature(raw_sig));

        let wrapper_sig = signature::construct_signature(wrapper_sig_bytes, &tx)?;
        tx.add_section(Section::Signature(wrapper_sig));

        tx.protocol_filter();

        Ok(tx)
    }

    /// Submit signed tx
    pub async fn submit_signed_tx(
        &mut self,
        tx_msg: &[u8],
        tx_bytes: &[u8],
        raw_sig_bytes: &[u8],
        wrapper_sig_bytes: &[u8],
    ) -> Result<(), JsError> {
        let transfer_tx = self.sign_tx(tx_bytes, raw_sig_bytes, wrapper_sig_bytes)?;
        let args = tx::tx_args_from_slice(tx_msg)?;
        let verification_key = args.verification_key.clone();
        let pk = validate_pk(verification_key)?;

        self.submit_reveal_pk(&args, &pk, None).await?;

        namada::ledger::tx::process_tx(&self.client, &mut self.wallet, &args, transfer_tx).await?;

        Ok(())
    }

    async fn signing_data(
        &mut self,
        address: Address,
        tx_args: args::Tx,
    ) -> Result<SigningTxData, Error> {
        let default_signer = signing::signer_from_address(Some(address.clone()));
        signing::aux_signing_data(
            &self.client,
            &mut self.wallet,
            &tx_args,
            &address,
            default_signer,
        )
        .await
    }

    pub async fn submit_transfer(
        &mut self,
        tx_msg: &[u8],
        password: Option<String>,
        xsk: Option<String>,
    ) -> Result<(), JsError> {
        let args = tx::transfer_tx_args(tx_msg, password, xsk)?;
        let signing_data = self
            .signing_data(args.source.effective_address(), args.tx.clone())
            .await?;

        let (tx_builder, _) = namada::ledger::tx::build_transfer(
            &self.client,
            &mut self.shielded_ctx,
            args.clone(),
            &signing_data.gas_payer,
        )
        .await?;

        self.sign_and_process_tx(args.tx, tx_builder, signing_data)
            .await?;

        Ok(())
    }

    pub async fn submit_ibc_transfer(
        &mut self,
        tx_msg: &[u8],
        password: Option<String>,
    ) -> Result<(), JsError> {
        let args = tx::ibc_transfer_tx_args(tx_msg, password)?;
        let signing_data = self
            .signing_data(args.source.clone(), args.tx.clone())
            .await?;

        let tx_builder = namada::ledger::tx::build_ibc_transfer(
            &self.client,
            args.clone(),
            &signing_data.gas_payer,
        )
        .await?;

        self.sign_and_process_tx(args.tx, tx_builder, signing_data)
            .await?;

        Ok(())
    }

    pub async fn submit_bond(
        &mut self,
        tx_msg: &[u8],
        password: Option<String>,
    ) -> Result<(), JsError> {
        let args = tx::bond_tx_args(tx_msg, password)?;
        let source = args.source.as_ref().expect("Source address is required");
        let signing_data = self.signing_data(source.clone(), args.tx.clone()).await?;

        let tx_builder =
            namada::ledger::tx::build_bond(&mut self.client, args.clone(), &signing_data.gas_payer)
                .await?;

        self.sign_and_process_tx(args.tx, tx_builder, signing_data)
            .await?;

        Ok(())
    }

    /// Submit unbond
    pub async fn submit_unbond(
        &mut self,
        tx_msg: &[u8],
        password: Option<String>,
    ) -> Result<(), JsError> {
        let args = tx::unbond_tx_args(tx_msg, password)?;
        let source = args.source.as_ref().expect("Source address is required");
        let signing_data = self.signing_data(source.clone(), args.tx.clone()).await?;

        let (tx_builder, _) = namada::ledger::tx::build_unbond(
            &mut self.client,
            &mut self.wallet,
            args.clone(),
            &signing_data.gas_payer,
        )
        .await?;

        self.sign_and_process_tx(args.tx, tx_builder, signing_data)
            .await?;

        Ok(())
    }

    pub async fn submit_withdraw(
        &mut self,
        tx_msg: &[u8],
        password: Option<String>,
    ) -> Result<(), JsError> {
        let args = tx::withdraw_tx_args(tx_msg, password)?;
        let source = args.source.as_ref().expect("Source address is required");
        let signing_data = self.signing_data(source.clone(), args.tx.clone()).await?;

        let tx_builder = namada::ledger::tx::build_withdraw(
            &mut self.client,
            args.clone(),
            &signing_data.gas_payer,
        )
        .await?;

        self.sign_and_process_tx(args.tx, tx_builder, signing_data)
            .await?;

        Ok(())
    }
}

#[wasm_bindgen(module = "/src/sdk/mod.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = "getMaspParams")]
    async fn get_masp_params() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = "hasMaspParams")]
    async fn has_masp_params() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = "fetchAndStoreMaspParams")]
    async fn fetch_and_store_masp_params() -> Result<JsValue, JsValue>;
}
