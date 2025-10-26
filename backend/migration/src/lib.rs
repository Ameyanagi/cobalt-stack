pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_initial_schema;
mod m20250125_000001_create_auth_tables;
mod m20250126_000001_add_email_verification_and_roles;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_initial_schema::Migration),
            Box::new(m20250125_000001_create_auth_tables::Migration),
            Box::new(m20250126_000001_add_email_verification_and_roles::Migration),
        ]
    }
}
