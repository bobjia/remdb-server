use crate::sql_engine::{ResultSet, SqlResult};
use remdb::RemDb;

pub struct InsertExecutor;

impl InsertExecutor {
    pub fn execute(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        let result = db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(result.rows.len()))
    }

    pub fn execute_batch(db: &mut RemDb, statements: &[String]) -> SqlResult<ResultSet> {
        let mut total_affected = 0;

        for stmt in statements {
            let result = db.sql_query(stmt)?;
            total_affected += result.rows.len();
        }

        Ok(ResultSet::with_affected_rows(total_affected))
    }
}
