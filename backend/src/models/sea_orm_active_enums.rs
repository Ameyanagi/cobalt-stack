//! SeaORM active enums for database enum types.
//!
//! This module defines Rust enums that map to PostgreSQL ENUM types.
//! These enums are used in entity fields to provide type-safe access
//! to database enum values.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// User role enum for role-based access control (RBAC).
///
/// Defines the authorization level for a user account.
/// Used in middleware to enforce access control policies.
///
/// # Variants
///
/// - `User`: Standard user with access to their own resources
/// - `Admin`: Administrator with elevated privileges
///
/// # Database Mapping
///
/// Maps to PostgreSQL ENUM `user_role` with values:
/// - `"user"` → [`UserRole::User`]
/// - `"admin"` → [`UserRole::Admin`]
///
/// # Examples
///
/// ```
/// use cobalt_stack::models::sea_orm_active_enums::UserRole;
///
/// let role = UserRole::User;
/// assert_eq!(serde_json::to_string(&role).unwrap(), "\"user\"");
///
/// let admin = UserRole::Admin;
/// assert_eq!(serde_json::to_string(&admin).unwrap(), "\"admin\"");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Standard user role with basic permissions.
    ///
    /// Users can:
    /// - Access their own profile
    /// - Update their own settings
    /// - View public resources
    #[sea_orm(string_value = "user")]
    User,

    /// Administrator role with elevated permissions.
    ///
    /// Admins can:
    /// - Access all user accounts
    /// - Disable/enable user accounts
    /// - View system-wide statistics
    /// - Perform administrative operations
    #[sea_orm(string_value = "admin")]
    Admin,
}
