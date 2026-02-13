use crate::sql_engine::{ResultSet, SqlResult};
use remdb::RemDb;

pub struct DdlExecutorHandler;

impl DdlExecutorHandler {
    pub fn execute_create_table(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(0))
    }

    pub fn execute_drop_table(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(0))
    }

    pub fn execute_alter_table(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(0))
    }

    pub fn execute_create_index(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(0))
    }

    pub fn execute_drop_index(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        db.sql_query(sql)?;
        Ok(ResultSet::with_affected_rows(0))
    }
}
