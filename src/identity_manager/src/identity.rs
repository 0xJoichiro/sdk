use crate::crypto_error::Error;
use crate::crypto_error::Result;
use crate::provider::basic::BasicProvider;
use crate::provider::Provider;
use crate::signature::Signature;

use std::path::PathBuf;

/// An identity is a construct that denotes the set of claims of an
/// entity about itself. Namely it collects principals, under which
/// the owner of this object can authenticate and provides basic
/// operations. Thus, an identity may have access to multiple
/// principals or credential services, each combination represented by
/// a provider.
pub struct Identity {
    // TODO(eftychis): This changes into a precendence map. Note that
    // in the future Principals are not going to be tied necessarily
    // with Identifiers from a canister's perspective.
    inner: Vec<Box<dyn Provider>>,
}

impl Identity {
    /// Return a corresponding provided a path.
    // Passing a simple configuration until we know all the necessary
    // configuration.
    pub fn new(path: PathBuf) -> Result<Self> {
        let basic_provider = BasicProvider::new(path)?;
        Ok(Self {
            inner: vec![Box::new(basic_provider)],
        })
    }
    /// Sign the provided message assuming a certain principal.
    pub fn sign(&self, msg: &[u8]) -> Result<Signature> {
        let provider = self.inner.first().ok_or(Error::NoProvider)?;
        let identity = provider
            .provide()
            .map_err(|_| Error::IdentityFailedToInitialize)?;
        identity.sign(msg)
    }
}
