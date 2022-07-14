# mysql-es

> A MySql implementation of the `PersistedEventRepository` trait in cqrs-es.

---

## Usage
Add to your Cargo.toml file:

```toml
[dependencies]
cqrs-es = "0.4.4"
mysql-es = "0.4.4"
```

Requires access to a MySql DB with existing tables. See:
- [Sample database configuration](db/init.sql)
- Use `docker-compose` to quickly setup [a local database](docker-compose.yml)

A simple configuration example:
```
let store = default_mysql_pool("mysql://my_user:my_pass@localhost:3306/my_db");
let cqrs = mysql_es::mysql_cqrs(pool, vec![])
```

Things that could be helpful:
- [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
- [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/master/docs/versions/change_log.md)

[![Crates.io](https://img.shields.io/crates/v/mysql-es)](https://crates.io/crates/mysql-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/mysql-es)
![docs](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoiRTZsVnY1emVCV1JXblVOMHpZTHdoS3JuVVVOUmRRb054Z2dYZmhKMk9PVU1zYklUaUhOTkM1d3l1czRWQUhBa28yWHM0RmRacmE3SWRmT1pJVU83akFVPSIsIml2UGFyYW1ldGVyU3BlYyI6InNuZ3U4MVBGYUFNbmhmLzIiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=main)
