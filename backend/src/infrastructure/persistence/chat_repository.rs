//! ChatRepository implementation using SeaORM
//!
//! Implements the domain ChatRepository trait for database persistence.

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::chat::{
        entity::{ChatMessage, ChatSession},
        repository::{ChatRepository, RepositoryError, RepositoryResult},
        value_objects::MessageRole,
    },
    models::{
        chat_messages, chat_sessions,
        prelude::{ChatMessages, ChatSessions},
    },
};

/// SeaORM implementation of ChatRepository
pub struct SeaOrmChatRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmChatRepository {
    /// Create a new repository instance
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Convert SeaORM model to domain entity
    fn model_to_session(model: chat_sessions::Model) -> ChatSession {
        ChatSession {
            id: model.id,
            user_id: model.user_id,
            title: model.title,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
            deleted_at: model.deleted_at.map(|dt| dt.with_timezone(&Utc)),
        }
    }

    /// Convert SeaORM model to domain entity
    fn model_to_message(model: chat_messages::Model) -> RepositoryResult<ChatMessage> {
        let role = MessageRole::from_str(&model.role)
            .map_err(|e| RepositoryError::ValidationError(e))?;

        Ok(ChatMessage {
            id: model.id,
            session_id: model.session_id,
            role,
            content: model.content,
            token_count: model.token_count,
            created_at: model.created_at.with_timezone(&Utc),
        })
    }
}

#[async_trait]
impl ChatRepository for SeaOrmChatRepository {
    async fn create_session(&self, session: &ChatSession) -> RepositoryResult<()> {
        let active_model = chat_sessions::ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            title: Set(session.title.clone()),
            created_at: Set(session.created_at.into()),
            updated_at: Set(session.updated_at.into()),
            deleted_at: Set(session.deleted_at.map(Into::into)),
        };

        active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_session_by_id(&self, id: Uuid) -> RepositoryResult<Option<ChatSession>> {
        let model = ChatSessions::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(model.map(Self::model_to_session))
    }

    async fn find_sessions_by_user(
        &self,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<(Vec<ChatSession>, u64)> {
        // Filter out deleted sessions
        let query = ChatSessions::find()
            .filter(chat_sessions::Column::UserId.eq(user_id))
            .filter(chat_sessions::Column::DeletedAt.is_null())
            .order_by_desc(chat_sessions::Column::CreatedAt);

        // Get total count
        let total = query
            .clone()
            .count(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Get paginated results
        let models = query
            .paginate(self.db.as_ref(), per_page)
            .fetch_page(page)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let sessions = models.into_iter().map(Self::model_to_session).collect();

        Ok((sessions, total))
    }

    async fn update_session(&self, session: &ChatSession) -> RepositoryResult<()> {
        let active_model = chat_sessions::ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            title: Set(session.title.clone()),
            created_at: Set(session.created_at.into()),
            updated_at: Set(Utc::now().into()),
            deleted_at: Set(session.deleted_at.map(Into::into)),
        };

        active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_session(&self, id: Uuid) -> RepositoryResult<()> {
        // Soft delete: set deleted_at timestamp
        let session = ChatSessions::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
            .ok_or(RepositoryError::SessionNotFound(id))?;

        let mut active_model: chat_sessions::ActiveModel = session.into();
        active_model.deleted_at = Set(Some(Utc::now().into()));

        active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn save_message(&self, message: &ChatMessage) -> RepositoryResult<()> {
        let active_model = chat_messages::ActiveModel {
            id: Set(message.id),
            session_id: Set(message.session_id),
            role: Set(message.role.as_str().to_string()),
            content: Set(message.content.clone()),
            token_count: Set(message.token_count),
            created_at: Set(message.created_at.into()),
        };

        active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_messages_by_session(
        &self,
        session_id: Uuid,
        limit: Option<u64>,
    ) -> RepositoryResult<Vec<ChatMessage>> {
        let mut query = ChatMessages::find()
            .filter(chat_messages::Column::SessionId.eq(session_id))
            .order_by_asc(chat_messages::Column::CreatedAt);

        if let Some(limit_value) = limit {
            query = query.limit(limit_value);
        }

        let models = query
            .all(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        models
            .into_iter()
            .map(Self::model_to_message)
            .collect::<RepositoryResult<Vec<_>>>()
    }

    async fn find_recent_messages(
        &self,
        session_id: Uuid,
        limit: u64,
    ) -> RepositoryResult<Vec<ChatMessage>> {
        // Get last N messages in descending order, then reverse to chronological
        let models = ChatMessages::find()
            .filter(chat_messages::Column::SessionId.eq(session_id))
            .order_by_desc(chat_messages::Column::CreatedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Reverse to get chronological order (oldest first)
        let mut messages = models
            .into_iter()
            .map(Self::model_to_message)
            .collect::<RepositoryResult<Vec<_>>>()?;

        messages.reverse();

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat::value_objects::MessageRole;

    #[test]
    fn test_model_to_session() {
        let model = chat_sessions::Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Test Session".to_string(),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
            deleted_at: None,
        };

        let session = SeaOrmChatRepository::model_to_session(model.clone());

        assert_eq!(session.id, model.id);
        assert_eq!(session.user_id, model.user_id);
        assert_eq!(session.title, model.title);
    }

    #[test]
    fn test_model_to_message() {
        let model = chat_messages::Model {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            token_count: Some(5),
            created_at: Utc::now().into(),
        };

        let message = SeaOrmChatRepository::model_to_message(model.clone()).unwrap();

        assert_eq!(message.id, model.id);
        assert_eq!(message.session_id, model.session_id);
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, model.content);
        assert_eq!(message.token_count, model.token_count);
    }

    #[test]
    fn test_model_to_message_invalid_role() {
        let model = chat_messages::Model {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            role: "invalid".to_string(),
            content: "Hello".to_string(),
            token_count: None,
            created_at: Utc::now().into(),
        };

        let result = SeaOrmChatRepository::model_to_message(model);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RepositoryError::ValidationError(_)
        ));
    }
}
