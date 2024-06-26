use crate::db::Database;
use crate::models::prelude::*;
use rusqlite::{named_params, Row};

pub struct InviteCode {
    pub code: String,
}

impl InviteCode {
    pub fn from_sql_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self { code: row.get(0)? })
    }

    pub async fn find(db: &Database, code: String) -> Result<Option<Self>> {
        let optional_code = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("SELECT code FROM invite_codes WHERE code = :code;")?;
                let mut rows = statement.query(named_params! {":code": code})?;
                match rows.next()? {
                    Some(row) => Ok(Some(Self::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_code)
    }

    pub async fn delete(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare("DELETE FROM invite_codes WHERE code = :code;")?;
                let result = statement.execute(named_params! {":code": self.code})?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}
