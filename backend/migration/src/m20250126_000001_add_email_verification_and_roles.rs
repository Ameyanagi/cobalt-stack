use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user_role enum type
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    CREATE TYPE user_role AS ENUM ('user', 'admin');
                EXCEPTION
                    WHEN duplicate_object THEN null;
                END $$;",
            )
            .await?;

        // Add role column to users table (nullable first for safe migration)
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::Role)
                            .custom(Alias::new("user_role"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Update existing users to have 'user' role
        manager
            .get_connection()
            .execute_unprepared("UPDATE users SET role = 'user' WHERE role IS NULL;")
            .await?;

        // Set default and make role NOT NULL
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE users
                 ALTER COLUMN role SET DEFAULT 'user'::user_role,
                 ALTER COLUMN role SET NOT NULL;",
            )
            .await?;

        // Add disabled_at column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::DisabledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add last_login_at column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::LastLoginAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes on new columns
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_role")
                    .table(Users::Table)
                    .col(Users::Role)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_disabled_at")
                    .table(Users::Table)
                    .col(Users::DisabledAt)
                    .to_owned(),
            )
            .await?;

        // Create email_verifications table
        manager
            .create_table(
                Table::create()
                    .table(EmailVerifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EmailVerifications::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_owned()),
                    )
                    .col(ColumnDef::new(EmailVerifications::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(EmailVerifications::TokenHash)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(EmailVerifications::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerifications::VerifiedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_verifications_user_id")
                            .from(EmailVerifications::Table, EmailVerifications::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes on email_verifications
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_email_verifications_user_id")
                    .table(EmailVerifications::Table)
                    .col(EmailVerifications::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_email_verifications_token_hash")
                    .table(EmailVerifications::Table)
                    .col(EmailVerifications::TokenHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_email_verifications_expires_at")
                    .table(EmailVerifications::Table)
                    .col(EmailVerifications::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop email_verifications table
        manager
            .drop_table(Table::drop().table(EmailVerifications::Table).to_owned())
            .await?;

        // Drop new columns from users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::LastLoginAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::DisabledAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Role)
                    .to_owned(),
            )
            .await?;

        // Drop user_role enum type
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS user_role;")
            .await?;

        Ok(())
    }
}

/// Table and column identifiers for users table additions
#[derive(DeriveIden)]
enum Users {
    Table,
    Role,
    DisabledAt,
    LastLoginAt,
    Id,
}

/// Table and column identifiers for email_verifications table
#[derive(DeriveIden)]
enum EmailVerifications {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    VerifiedAt,
    CreatedAt,
}
