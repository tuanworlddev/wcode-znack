//! Static configuration. Production endpoints only (per requirements).

pub const CONTENT_BASE: &str = "https://content-api.wildberries.ru";
pub const MARKETPLACE_BASE: &str = "https://marketplace-api.wildberries.ru";

/// Keyring service used to store WB API tokens (account = `wb_token_<storeId>`).
pub const KEYRING_SERVICE: &str = "com.rupphi.wcode";
