/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Provides struct field versions of the thread local stats.
//!
use std::cell::RefCell;
use std::collections::HashMap;
use std::thread::LocalKey;

use crate::stat_types::BoxCounter;
use crate::stat_types::BoxHistogram;
use crate::stat_types::BoxTimeseries;
use crate::stat_types::Counter;
use crate::stat_types::Histogram;
use crate::stat_types::Timeseries;

pub struct FieldStatThreadLocal<TStatType: 'static> {
    map: RefCell<HashMap<String, TStatType>>,
    stat_generator: fn(&str) -> TStatType,
}

impl<TStatType> FieldStatThreadLocal<TStatType> {
    pub fn new(stat_generator: fn(&str) -> TStatType) -> Self {
        FieldStatThreadLocal {
            map: RefCell::new(HashMap::new()),
            stat_generator,
        }
    }

    fn get_or_default<F, V>(&self, key: &str, cb: F) -> V
    where
        F: FnOnce(&TStatType) -> V,
    {
        let mut map = self.map.borrow_mut();
        match map.get(key) {
            Some(stat) => cb(stat),
            None => {
                let stat = (self.stat_generator)(key);
                let v = cb(&stat);
                map.insert(key.to_string(), stat);
                v
            }
        }
    }
}

pub struct FieldStat<TStatType: 'static> {
    tl: &'static LocalKey<FieldStatThreadLocal<TStatType>>,
    key: String,
}

impl<TStatType> FieldStat<TStatType> {
    pub fn new(tl: &'static LocalKey<FieldStatThreadLocal<TStatType>>, key: String) -> Self {
        FieldStat { tl, key }
    }
}

impl Counter for FieldStat<BoxCounter> {
    fn increment_value(&self, value: i64) {
        self.tl
            .with(|tl| tl.get_or_default(&self.key, |s| s.increment_value(value)));
    }
}

impl Timeseries for FieldStat<BoxTimeseries> {
    fn add_value(&self, value: i64) {
        self.tl
            .with(|tl| tl.get_or_default(&self.key, |s| s.add_value(value)));
    }

    fn add_value_aggregated(&self, value: i64, nsamples: u32) {
        self.tl
            .with(|tl| tl.get_or_default(&self.key, |s| s.add_value_aggregated(value, nsamples)));
    }
}

impl Histogram for FieldStat<BoxHistogram> {
    fn add_value(&self, value: i64) {
        self.tl
            .with(|tl| tl.get_or_default(&self.key, |s| s.add_value(value)));
    }

    fn add_repeated_value(&self, value: i64, nsamples: u32) {
        self.tl
            .with(|tl| tl.get_or_default(&self.key, |s| s.add_repeated_value(value, nsamples)));
    }
}
