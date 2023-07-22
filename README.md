# PING PONG, STOP
A concurrent PING / PONG server with a remote shutdown command, built in Rust with the Tokio runtime


**Start server:**
```
cargo run
```

**Connect:**
```
telnet 127.0.0.1:8080
```

**Commands:** `PING`, `STOP`
**Responses:** `PONG`, `SURE`
