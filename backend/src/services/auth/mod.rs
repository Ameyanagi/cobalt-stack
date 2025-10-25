pub mod error;
pub mod jwt;
pub mod password;
pub mod token_rotation;

pub use error::{AuthError, Result};
pub use jwt::{
    create_access_token, create_refresh_token, verify_access_token, verify_refresh_token,
    AccessTokenClaims, JwtConfig, RefreshTokenClaims,
};
pub use password::{hash_password, verify_password};
pub use token_rotation::{
    cleanup_expired_tokens, revoke_all_user_tokens, revoke_refresh_token, rotate_refresh_token,
    store_refresh_token, validate_refresh_token,
};
