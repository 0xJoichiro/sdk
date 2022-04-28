use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::provider::create_agent_environment;

use anyhow::anyhow;
use clap::Parser;
use std::str::FromStr;
use tokio::runtime::Runtime;

mod account_id;
mod balance;
mod create_canister;
mod fabricate_cycles;
mod notify;
mod top_up;
mod transfer;

/// Ledger commands.
#[derive(Parser)]
#[clap(name("ledger"))]
pub struct LedgerOpts {
    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    AccountId(account_id::AccountIdOpts),
    Balance(balance::BalanceOpts),
    CreateCanister(create_canister::CreateCanisterOpts),
    FabricateCycles(fabricate_cycles::FabricateCyclesOpts),
    Notify(notify::NotifyOpts),
    TopUp(top_up::TopUpOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, opts: LedgerOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::AccountId(v) => account_id::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::CreateCanister(v) => create_canister::exec(&agent_env, v).await,
            SubCommand::FabricateCycles(v) => fabricate_cycles::exec(&agent_env, v).await,
            SubCommand::Notify(v) => notify::exec(&agent_env, v).await,
            SubCommand::TopUp(v) => top_up::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
        }
    })
}

fn get_icpts_from_args(
    amount: &Option<String>,
    icp: &Option<String>,
    e8s: &Option<String>,
) -> DfxResult<ICPTs> {
    match amount {
        None => {
            let icp = match icp {
                Some(s) => {
                    // validated by e8s_validator
                    let icps = s.parse::<u64>().unwrap();
                    ICPTs::from_icpts(icps).map_err(|err| anyhow!(err))?
                }
                None => ICPTs::from_e8s(0),
            };
            let icp_from_e8s = match e8s {
                Some(s) => {
                    // validated by e8s_validator
                    let e8s = s.parse::<u64>().unwrap();
                    ICPTs::from_e8s(e8s)
                }
                None => ICPTs::from_e8s(0),
            };
            let amount = icp + icp_from_e8s;
            Ok(amount.map_err(|err| anyhow!(err))?)
        }
        Some(amount) => Ok(ICPTs::from_str(amount)
            .map_err(|err| anyhow!("Could not add ICPs and e8s: {}", err))?),
    }
}
