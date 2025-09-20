use std::sync::Arc;
use tokio_rusqlite::{Connection, params};

#[derive(Clone, Debug)]
pub struct ListItem {
    pub id: u64,
    pub name: String,
}

pub struct ListRepo {
    conn: Arc<Connection>,
}

impl ListRepo {
    pub async fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::open(db_path).await?;

        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS list_items (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                )",
                [],
            )?;
            Ok(())
        }).await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn add_item(&self, name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if name.len() > 100 {
            return Err("Name is too long (max 100 characters)".to_string());
        }

        let name = name.to_string();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO list_items (name) VALUES (?1)",
                    params![name],
                )?;
                Ok(())
            })
            .await
            .map_err(|e| format!("Failed to add item: {}", e))
    }

    pub async fn remove_item(&self, id: u64) -> Result<(), String> {
        let affected = self.conn
            .call(move |conn| {
                let affected = conn.execute(
                    "DELETE FROM list_items WHERE id = ?1",
                    params![id as i64],
                )?;
                Ok(affected)
            })
            .await
            .map_err(|e| format!("Failed to remove item: {}", e))?;

        if affected == 0 {
            Err(format!("Item with id {} not found", id))
        } else {
            Ok(())
        }
    }

    pub async fn clear(&self) -> Result<(), String> {
        self.conn
            .call(|conn| {
                conn.execute("DELETE FROM list_items", [])?;
                Ok(())
            })
            .await
            .map_err(|e| format!("Failed to clear list: {}", e))
    }

    pub async fn list(&self) -> Result<Vec<ListItem>, String> {
        self.conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT id, name FROM list_items ORDER BY id")?;
                let items = stmt
                    .query_map([], |row| {
                        Ok(ListItem {
                            id: row.get::<_, i64>(0)? as u64,
                            name: row.get(1)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(items)
            })
            .await
            .map_err(|e| format!("Failed to list items: {}", e))
    }
}