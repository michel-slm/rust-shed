/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Contains basic definitions for the sql crate and for any crate that wish
//! to implement traits to be used with the sql's queries macro.

#![deny(warnings, missing_docs, clippy::all, intra_doc_link_resolution_failure)]

pub mod error;
pub mod ext;
pub mod mysql;
pub mod mysql2;
pub mod sqlite;
pub mod transaction;

use std::fmt::{self, Debug};
use std::sync::Arc;

// Used in docs
#[cfg(test)]
mod _unused {
    use sql as _;
    use sql_tests_lib as _;
}

/// Enum that generalizes over connections to Sqlite and MyRouter.
#[derive(Clone)]
pub enum Connection {
    /// Sqlite lets you use this crate with rusqlite connections such as in memory or on disk Sqlite
    /// databases, both useful in case of testing or local sql db use cases.
    Sqlite(Arc<sqlite::SqliteMultithreaded>),
    /// An enum variant for the mysql-based connections, your structure have to
    /// implement [mysql::MysqlConnection] in order to be usable here.
    Mysql(mysql::BoxMysqlConnection),
}

impl From<sqlite::SqliteMultithreaded> for Connection {
    fn from(con: sqlite::SqliteMultithreaded) -> Self {
        Connection::Sqlite(Arc::new(con))
    }
}

impl From<mysql::BoxMysqlConnection> for Connection {
    fn from(con: mysql::BoxMysqlConnection) -> Self {
        Connection::Mysql(con)
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Connection::Sqlite(..) => write!(f, "Sqlite"),
            Connection::Mysql(ref con) => con.fmt(f),
        }
    }
}

/// Value returned from a `write` type of query
pub struct WriteResult {
    last_insert_id: Option<u64>,
    affected_rows: u64,
}

impl WriteResult {
    /// Method made public for access from inside macros, you probably don't want to use it.
    pub fn new(last_insert_id: Option<u64>, affected_rows: u64) -> Self {
        WriteResult {
            last_insert_id,
            affected_rows,
        }
    }

    /// Return the id of last inserted row if any.
    pub fn last_insert_id(&self) -> Option<u64> {
        self.last_insert_id
    }

    /// Return number of rows affected by the `write` query
    pub fn affected_rows(&self) -> u64 {
        self.affected_rows
    }
}
