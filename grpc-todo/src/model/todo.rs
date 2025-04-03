use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, prelude::FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

pub async fn create_todo(pool: &Pool<Postgres>, title: &str) -> Todo {
    sqlx::query_as(
        "
            INSERT INTO todos (title, completed) VALUES ($1, false) RETURNING *
            ",
    )
    .bind(title)
    .fetch_one(pool)
    .await
    .expect("Failed to create todo")
}

pub async fn get_todos(pool: &Pool<Postgres>) -> Vec<Todo> {
    sqlx::query_as(
        "
            SELECT * FROM todos
            ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get todos")
}

pub async fn delete_todo(pool: &Pool<Postgres>, id: i64) -> Todo {
    sqlx::query_as(
        "
            DELETE FROM todos WHERE id = $1 RETURNING *
            ",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .expect("Failed to delete todo")
}

pub async fn update_todo(pool: &Pool<Postgres>, id: i64, title: &str, completed: bool) -> Todo {
    sqlx::query_as(
        "
            UPDATE todos SET title = $1, completed = $2 WHERE id = $3 RETURNING *
            ",
    )
    .bind(title)
    .bind(completed)
    .bind(id)
    .fetch_one(pool)
    .await
    .expect("Failed to update todo")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_new_for_test;
    use anyhow::Result;

    #[tokio::test]
    async fn create_todo_should_work() -> Result<()> {
        let (_tdb, pool) = db_new_for_test().await?;
        let title = "Hello World";

        let todo = create_todo(&pool, title).await;

        assert_eq!(todo.title, title);
        assert!(!todo.completed);

        Ok(())
    }

    #[tokio::test]
    async fn delete_todo_should_work() -> Result<()> {
        let (_tdb, pool) = db_new_for_test().await?;

        let todo = delete_todo(&pool, 1).await;

        assert_eq!(todo.title, "Hello Test");
        assert_eq!(todo.id, 1);
        assert!(!todo.completed);

        Ok(())
    }

    #[tokio::test]
    async fn update_todo_should_work() -> Result<()> {
        let (_tdb, pool) = db_new_for_test().await?;

        let todo = update_todo(&pool, 1, "Hello World", true).await;

        assert_eq!(todo.title, "Hello World");
        assert_eq!(todo.id, 1);
        assert!(todo.completed);

        Ok(())
    }

    #[tokio::test]
    async fn get_todos_should_work() -> Result<()> {
        let (_tdb, pool) = db_new_for_test().await?;

        let todos = get_todos(&pool).await;

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Hello Test");
        assert_eq!(todos[0].id, 1);
        assert!(!todos[0].completed);

        Ok(())
    }
}
