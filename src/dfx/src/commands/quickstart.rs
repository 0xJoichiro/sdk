use anyhow::Error;
use candid::Principal;
use dialoguer::{Confirm, Input};
use ic_agent::Identity as _;
use ic_utils::interfaces::{WalletCanister, ManagementCanister, management_canister::builders::InstallMode};
use num_traits::Inv;
use rust_decimal::Decimal;
use tokio::runtime::Runtime;

use crate::{lib::{
    environment::Environment,
    error::DfxResult,
    identity::{IdentityManager, Identity},
    nns_types::account_identifier::AccountIdentifier,
    operations::ledger::{balance, icp_xdr_rate},
    provider::create_agent_environment, waiter::waiter_with_timeout,
}, util::{assets::wallet_wasm, expiry_duration}};

pub fn exec(env: &dyn Environment) -> DfxResult {
    let env = create_agent_environment(env, Some("ic".to_string()))?;
    let agent = env.get_agent().expect("Unable to create agent");
    let ident = IdentityManager::new(&env)?.instantiate_selected_identity()?;
    let principal = ident.sender().map_err(Error::msg)?;
    println!("Your DFX user principal: {principal}");
    let acct = AccountIdentifier::new(principal, None);
    println!("Your ledger account ID: {acct}");
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        let balance = balance(agent, &acct, None).await?;
        println!("Your ICP balance: {balance}");
        let xdr_conversion_rate = icp_xdr_rate(agent).await?;
        let xdr_per_icp = Decimal::from_i128_with_scale(xdr_conversion_rate as i128, 4);
        let icp_per_tc = xdr_per_icp.inv();
        println!("Conversion rate: 1 ICP <> {xdr_per_icp} XDR");
        let wallet = Identity::wallet_canister_id(&env, env.get_network_descriptor().unwrap(), ident.name());
        if let Ok(wallet) = wallet {
            println!("Mainnet wallet canister: {wallet}");
            if let Ok(wallet_canister) = WalletCanister::create(agent, wallet).await {
                if let Ok(balance) = wallet_canister.wallet_balance().await {
                    println!("Mainnet wallet balance: {:.2} TC", Decimal::from(balance.amount) / Decimal::from(1_000_000_000_000_u64));
                }
            }
        } else if Confirm::new().with_prompt("Import an existing wallet?").interact()? {
            let id = Input::<Principal>::new().with_prompt("Paste the principal ID of the existing wallet:").interact_text()?;
            let wallet = if let Ok(wallet) = WalletCanister::create(agent, id).await {
                wallet
            } else {
                let mgmt = ManagementCanister::create(agent);
                let wasm = wallet_wasm(env.get_logger())?;
                mgmt.install_code(&id, &wasm).with_mode(InstallMode::Install).call_and_wait(waiter_with_timeout(expiry_duration())).await?;
                WalletCanister::create(agent, id).await?
            };
            Identity::set_wallet_id(&env, env.get_network_descriptor().unwrap(), ident.name(), id)?;
            println!("Successfully imported wallet {id}.");
            if let Ok(balance) = wallet.wallet_balance().await {
                println!("Mainnet wallet balance: {:.2} TC", Decimal::from(balance.amount) / Decimal::from(1_000_000_000_000_u64));
            }
        } else {
            let possible_tc = xdr_per_icp * balance.to_decimal();
            let needed_tc = Decimal::new(10, 0) - possible_tc;
            if needed_tc.is_sign_positive() {
                let needed_icp = needed_tc * icp_per_tc;
                println!("You need {needed_icp} more ICP to deploy a 10 TC wallet canister on mainnet.");
                println!("Deposit {needed_icp} ICP into the address {acct}, and then run this command again, to deploy a mainnet wallet.");
                println!("Alternatively:");
                println!("- If you have ICP in an NNS account, you can create a new canister through the NNS interface");
                println!("- If you have a Twitter account, you can link it to the cycles faucet to get free cycles at https://faucet.dfinity.org");
                println!("Either of these options will ask for your DFX user principal, listed above.");
                println!("And either of these options will hand you back a wallet canister principal; when you run the command again, select the 'import an existing wallet' option.");
            } else {
                if Confirm::new().with_prompt(format!("Spend {} ICP to create a new wallet with 10 TC?", Decimal::new(10, 0) * icp_per_tc)).interact()? {
                    // todo blocked on ledger change
                } else {
                    println!("Run this command again at any time to continue from here."); // unify somehow
                    return Ok(());
                }
            }
        }
        Ok(())
    })
}
