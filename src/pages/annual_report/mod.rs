pub mod utils;
pub mod components;
pub mod sections;
pub mod page;

use yew::prelude::*;
use crate::wallet::WalletAccount;

#[derive(Properties, PartialEq, Clone)]
pub struct AnnualReportPageProps {
    pub fid: i64,
    pub api_url: String,
    pub wallet_account: Option<WalletAccount>,
    pub is_farcaster_env: bool,
    pub share_url: Option<String>,
    pub current_user_fid: Option<i64>,
}

pub use components::*;
pub use page::AnnualReportPage;

