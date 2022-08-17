use crate::DfxResult;

use anyhow::{bail, Context};
use ic_agent::Agent;
use std::path::Path;

pub async fn install_nns(
    _agent: &Agent,
    ic_nns_init_path: &Path,
    _replicated_state_dir: &Path,
) -> DfxResult {
    // Notes:
    //   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different binary for local development
    //   - this won't work with an HSM, because the agent holds a session open

    let mut cmd = std::process::Command::new(ic_nns_init_path);
    cmd.arg("--help");
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let output = cmd
        .output()
        .with_context(|| format!("Error executing {:#?}", cmd))?;

    if !output.status.success() {
        bail!("ic-nns-init call failed");
    }
    Ok(())
}