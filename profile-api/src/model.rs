use std::collections::HashMap;

use anyhow::{anyhow, Result, Ok};
use serde::{Serialize, Deserialize};
use spin_sdk::http::HeaderValue;

use spin_sdk::pg::{self as db, Decode, ParameterValue, Row};

use crate::utils::{get_last_param_from_route, get_column_lookup};

fn as_param<'a>(value: &Option<String>) -> Option<ParameterValue> {
    match value {
        Some(value) => Some(ParameterValue::Str(value.clone())),
        None => None
    }
}

fn as_nullable_param<'a>(value: &Option<String>) -> ParameterValue {
    as_param(value).unwrap_or_else(|| ParameterValue::DbNull)
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Profile {
    pub id: Option<String>,
    pub handle: String,
    pub avatar: Option<String>,
}

impl Profile {
    pub(crate) fn from_path(header: &Option<&HeaderValue>) -> Result<Self> {
        let header = header.ok_or(anyhow!("Error: Failed to discover path"))?;
        let path = header.as_str().ok_or(anyhow!("Error: Failed to convert path"))?;
        match get_last_param_from_route(path) {
            Some(handle) => Ok(Profile {
                id: None,
                handle,
                avatar: None,
            }),
            None => Err(anyhow!("Failed to parse handle from path")),
        }
    }

    pub(crate) fn from_bytes(b: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(&b)?)
    }

    fn from_row(row: &Row, columns: &HashMap<&str, usize>) -> Result<Self> {
        let id = String::decode(&row[columns["id"]]).ok();
        let handle = String::decode(&row[columns["handle"]])?;
        let avatar = String::decode(&row[columns["avatar"]]).ok();
        Ok(Profile {
            id,
            handle,
            avatar,
        })
    }

    pub(crate) fn insert(&self, db_url: &str) -> Result<()> {
        let params = vec![
            as_param(&self.id).ok_or(anyhow!("The id field is currently required for insert"))?,
            ParameterValue::Str((&self.handle).parse()?),
            as_param(&self.avatar).unwrap_or_else(|| ParameterValue::DbNull)
        ];
        let db_connection = db::Connection::open(db_url)?;
        db::Connection::execute(
            &db_connection,
            "INSERT INTO profiles (id, handle, avatar) VALUES ($1, $2, $3)",
            &params
        )?;
        Ok(())
    }

    pub(crate) fn get_by_handle(handle: &str, db_url: &str) -> Result<Profile> {
        let params = vec![ParameterValue::Str(handle.to_string())];
        let db_connection = db::Connection::open(db_url)?;
        let row_set = db::Connection::query(
            &db_connection,
            "SELECT id, handle, avatar from profiles WHERE handle=$1",
            &params
        )?;
        let columns = get_column_lookup(&row_set.columns);

        match row_set.rows.first() {
            Some(row) => Profile::from_row(row, &columns),
            None => Err(anyhow!("Profile not found for handle '{:?}'", handle))
        }
    }

    pub(crate) fn update(&self, db_url: &str) -> Result<()> {
        let db_connection = db::Connection::open(db_url)?;
        match &self.id {
            Some(id) => {
                let params = vec![
                    ParameterValue::Str((&self.handle).parse()?),
                    as_nullable_param(&self.avatar),
                    ParameterValue::Str(id.to_string()),
                ];
                db::Connection::execute(
                    &db_connection,
                    "UPDATE profiles SET handle=$1, avatar=$2 WHERE id=$3",
                    &params
                )?;
            },
            None => {
                let params = vec![
                    as_nullable_param(&self.avatar),
                    ParameterValue::Str(self.handle.to_string())
                ];
                db::Connection::execute(
                    &db_connection,
                    "UPDATE profiles SET avatar=$1 WHERE handle=$2",
                    &params
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn delete(&self, db_url: &str) -> Result<()> {
        let db_connection = db::Connection::open(db_url)?;
        match &self.id {
            Some(id) => {
                let params = vec![
                    ParameterValue::Str(id.to_string())
                ];
                db::Connection::execute(
                    &db_connection,
                    "DELETE FROM profiles WHERE id=$1",
                    &params
                )?;
            },
            None => {
                let params = vec![
                    ParameterValue::Str(self.handle.to_string())
                ];
                db::Connection::execute(
                    &db_connection,
                    "DELETE FROM profiles WHERE handle=$1",
                    &params
                )?;
            }
        }
        Ok(())
    }
}