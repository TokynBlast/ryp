pub enum MarketResult {
    Error(String),
    Results(Vec<crate::app::marketplace::MarketplacePlugin>),
    OnlineSet(bool),
}

#[derive(serde::Deserialize)]
pub struct MarketplacePlugin {
    pub title: String,
    pub desc: String,
}
