use crate::sql_engine::{ResultSet, SqlResult};
use remdb::RemDb;

pub struct UpdateExecutor;

impl UpdateExecutor {
    pub fn execute(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        let result = db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(result.rows.len()))
    }
}
