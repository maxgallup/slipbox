extern crate slipbox_core;


use std::path::PathBuf;

use slipbox_core::{init_tracing, Result, Vault};
use tracing::info;


fn main() -> Result<()> {
    init_tracing();

    let vault = Vault::new(PathBuf::from("./slipbox-core/tests/vault"))?;

    info!("{:#?}", vault);


    Ok(())
}