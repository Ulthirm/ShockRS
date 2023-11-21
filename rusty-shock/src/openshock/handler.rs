// https://api.shocklink.net/swagger/index.html
// https://github.com/OpenShock
// API Test Auth token ziaMvnhog1l3v9W2pnLqjo7XthKwJK7dV4HVh73NJWieMfsidAKFlwkrm3GrO8y6
use crate::{config,osc::touchpoints::CommandState};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

pub async fn handler(websocket_url: String,commandmap: Arc<Mutex<HashMap<String, CommandState>>>) {
    log::debug!("Openshock Touchpoint Handler");
    log::debug!("WebSocket URL: {}",websocket_url);
    let firmware_config = &config::get_config().firmware;

    // get the auth token from the config
    let auth_token = &firmware_config.api_authtoken;
    // verify the auth token is not blank or other error before dispatching to endpoints/auth.rs
    if auth_token.is_empty() {
        log::error!("Auth token is blank. Please check your config.toml file.");
        return;
    } else {
        log::debug!("Auth token is not blank. Continuing.");
    }

}