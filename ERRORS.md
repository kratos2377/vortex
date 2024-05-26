# Error Tracking

### Getting following error when user sends user-connection-event in the beginning
```thread 'tokio-runtime-worker' panicked at crates\messier\src\event_producer\user_events_producer.rs:94:95:
called `Result::unwrap()` on an `Err` value: Query(SqlxError(Database(PgDatabaseError { severity: Error, code: "42703", message: "column users_friends.userid does not exist", detail: None, hint: Some("Perhaps you meant to reference the column \"users_friends.user_id\"."), position: Some(Original(30)), where: None, schema: None, table: None, column: None, data_type: None, constraint: None, file: Some("parse_relation.c"), line: Some(3720), routine: Some("errorMissingColumn") })))```

### Deserialization Error from get_ongoing_games_users
```
Error { kind: BsonDeserialization(DeserializationError { message: "missing field `player_status`" }), labels: {}, wire_version: None, source: None }
```