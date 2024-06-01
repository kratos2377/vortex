# Error Tracking


### Some Kafka Issue
```
thread 'tokio-runtime-worker' panicked at crates\messier\src\event_producer\user_events_producer.rs:52:34:
called `Result::unwrap()` on an `Err` value: KafkaError (Transaction error: Operation not valid in state InTransaction)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'tokio-runtime-worker' panicked at crates\messier\src\event_producer\user_events_producer.rs:52:34:
called `Result::unwrap()` on an `Err` value: KafkaError (Transaction error: Operation not valid in state InTransaction)
```