use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create chat_sessions table
        manager
            .create_table(
                Table::create()
                    .table(ChatSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatSessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_owned()),
                    )
                    .col(ColumnDef::new(ChatSessions::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ChatSessions::Title)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned()),
                    )
                    .col(
                        ColumnDef::new(ChatSessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned()),
                    )
                    .col(
                        ColumnDef::new(ChatSessions::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_sessions_user_id")
                            .from(ChatSessions::Table, ChatSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on user_id for efficient session listing
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_chat_sessions_user_id")
                    .table(ChatSessions::Table)
                    .col(ChatSessions::UserId)
                    .to_owned(),
            )
            .await?;

        // Create index on created_at for ordering sessions
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_chat_sessions_created_at")
                    .table(ChatSessions::Table)
                    .col(ChatSessions::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Create chat_messages table
        manager
            .create_table(
                Table::create()
                    .table(ChatMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatMessages::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_owned()),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::SessionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::Role)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChatMessages::Content).text().not_null())
                    .col(ColumnDef::new(ChatMessages::TokenCount).integer().null())
                    .col(
                        ColumnDef::new(ChatMessages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_messages_session_id")
                            .from(ChatMessages::Table, ChatMessages::SessionId)
                            .to(ChatSessions::Table, ChatSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add CHECK constraint on role (must be 'user', 'assistant', or 'system')
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE chat_messages ADD CONSTRAINT chat_messages_role_check \
                 CHECK (role IN ('user', 'assistant', 'system'))",
            )
            .await?;

        // Create index on session_id for efficient message retrieval
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_chat_messages_session_id")
                    .table(ChatMessages::Table)
                    .col(ChatMessages::SessionId)
                    .to_owned(),
            )
            .await?;

        // Create index on created_at for ordering messages
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_chat_messages_created_at")
                    .table(ChatMessages::Table)
                    .col(ChatMessages::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order due to foreign key constraints
        manager
            .drop_table(Table::drop().table(ChatMessages::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ChatSessions::Table).to_owned())
            .await?;

        Ok(())
    }
}

/// Table and column identifiers for chat_sessions table
#[derive(DeriveIden)]
enum ChatSessions {
    Table,
    Id,
    UserId,
    Title,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

/// Table and column identifiers for chat_messages table
#[derive(DeriveIden)]
enum ChatMessages {
    Table,
    Id,
    SessionId,
    Role,
    Content,
    TokenCount,
    CreatedAt,
}

/// Table and column identifiers for users table (for foreign key)
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
