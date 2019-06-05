use serde::{Deserialize, Serialize};

use crate::wallet;

#[derive(Debug, Deserialize)]
pub struct WalletInfosRequest;

#[derive(Debug, Serialize)]
pub struct WalletInfosResponse {
    pub infos: Vec<wallet::WalletInfo>,
}
