use {
    jsonrpc_core::{Error, Result},
    safecoin_account_decoder::{
        parse_account_data::AccountAdditionalData,
        parse_token::{
            get_token_account_mint, safe_token_native_mint, safe_token_native_mint_program_id,
        },
        UiAccount, UiAccountData, UiAccountEncoding,
    },
    safecoin_client::rpc_response::RpcKeyedAccount,
    solana_runtime::bank::Bank,
    safecoin_sdk::{
        account::{AccountSharedData, ReadableAccount},
        pubkey::Pubkey,
    },
    safe_token_2022::{extension::StateWithExtensions, state::Mint},
    std::{collections::HashMap, sync::Arc},
};

pub fn get_parsed_token_account(
    bank: Arc<Bank>,
    pubkey: &Pubkey,
    account: AccountSharedData,
) -> UiAccount {
    let additional_data = get_token_account_mint(account.data())
        .and_then(|mint_pubkey| get_mint_owner_and_decimals(&bank, &mint_pubkey).ok())
        .map(|(_, decimals)| AccountAdditionalData {
            safe_token_decimals: Some(decimals),
        });

    UiAccount::encode(
        pubkey,
        &account,
        UiAccountEncoding::JsonParsed,
        additional_data,
        None,
    )
}

pub fn get_parsed_token_accounts<I>(
    bank: Arc<Bank>,
    keyed_accounts: I,
) -> impl Iterator<Item = RpcKeyedAccount>
where
    I: Iterator<Item = (Pubkey, AccountSharedData)>,
{
    let mut mint_decimals: HashMap<Pubkey, u8> = HashMap::new();
    keyed_accounts.filter_map(move |(pubkey, account)| {
        let additional_data = get_token_account_mint(account.data()).map(|mint_pubkey| {
            let safe_token_decimals = mint_decimals.get(&mint_pubkey).cloned().or_else(|| {
                let (_, decimals) = get_mint_owner_and_decimals(&bank, &mint_pubkey).ok()?;
                mint_decimals.insert(mint_pubkey, decimals);
                Some(decimals)
            });
            AccountAdditionalData { safe_token_decimals }
        });

        let maybe_encoded_account = UiAccount::encode(
            &pubkey,
            &account,
            UiAccountEncoding::JsonParsed,
            additional_data,
            None,
        );
        if let UiAccountData::Json(_) = &maybe_encoded_account.data {
            Some(RpcKeyedAccount {
                pubkey: pubkey.to_string(),
                account: maybe_encoded_account,
            })
        } else {
            None
        }
    })
}

/// Analyze a mint Pubkey that may be the native_mint and get the mint-account owner (token
/// program_id) and decimals
pub fn get_mint_owner_and_decimals(bank: &Arc<Bank>, mint: &Pubkey) -> Result<(Pubkey, u8)> {
    if mint == &safe_token_native_mint() {
        Ok((
            safe_token_native_mint_program_id(),
            safe_token::native_mint::DECIMALS,
        ))
    } else {
        let mint_account = bank.get_account(mint).ok_or_else(|| {
            Error::invalid_params("Invalid param: could not find mint".to_string())
        })?;
        let decimals = get_mint_decimals(mint_account.data())?;
        Ok((*mint_account.owner(), decimals))
    }
}

fn get_mint_decimals(data: &[u8]) -> Result<u8> {
    StateWithExtensions::<Mint>::unpack(data)
        .map_err(|_| {
            Error::invalid_params("Invalid param: Token mint could not be unpacked".to_string())
        })
        .map(|mint| mint.base.decimals)
}
