pub mod components;
pub mod page;
pub mod sections;
pub mod utils;

use yew::prelude::*;

use crate::farcaster;
use crate::wallet::WalletAccount;

#[derive(Properties, PartialEq, Clone)]
pub struct AnnualReportPageProps {
    pub fid: i64,
    pub api_url: String,
    pub wallet_account: Option<WalletAccount>,
    pub is_farcaster_env: bool,
    pub share_url: Option<String>,
    pub current_user_fid: Option<i64>,
    pub farcaster_context: Option<farcaster::MiniAppContext>,
}

pub use components::*;
pub use page::AnnualReportPage;
