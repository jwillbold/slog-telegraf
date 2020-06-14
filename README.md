# slog-telegraf

[slog-rs](https://github.com/slog-rs/slog) drain for [Telegraf](https://www.influxdata.com/time-series-platform/telegraf/).

## Usage

Currently, only the tcp socket is supported. If you need UDP, UNIX socket or something else, feel free to open an issue or pull request.
### Telegraf setup
```conf
[[inputs.socket_listener]]
  service_address = "tcp://localhost:8094"
```

### Rust setup

```Rust
use slog::{Logger, Drain};
use slog_telegraf::{TelegrafDrain};

fn main() {
    let drain = TelegrafDrain::new("tcp://192.168.0.108:8094".into(), "test".into()).unwrap().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let log = Logger::root(drain, o!("ver" => "1.2.1"));
    info!(log, "log"; "key" => 10);
}
```

### Notes
The only values treated as fields, are the values passed in the logging call. In the example above, ``key=10i`` is a field.
All other values are treated as tags. In the example above, ``msg="log",mod="your_crate::main",ver="1.2.1"`` are tags.