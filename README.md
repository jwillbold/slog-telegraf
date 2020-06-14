# slog-telegraf

[slog-rs](https://github.com/slog-rs/slog) drain for [Telegraf](https://www.influxdata.com/time-series-platform/telegraf/).
Formats the log message and sends it using TCP to Telegraf. 

Feel free to open issues or pull requests.

## Usage

The logger supports the TCP and UDP socket listener of Telegraf.

### Telegraf setup
```conf
[[inputs.socket_listener]]
  service_address = "tcp://localhost:8094"
```

or

```conf
[[inputs.socket_listener]]
  service_address = "udp://localhost:8094"
```

### Rust example

```Rust
use slog::{Logger, Drain};
use slog_telegraf::{TelegrafDrain};

fn main() {
    let drain = TelegrafDrain::new("tcp://192.168.0.108:8094".into(), "measurement".into()).unwrap().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let log = Logger::root(drain, o!("ver" => "1.2.1"));
    info!(log, "log"; "field_key" => 10);
}
```

## Notes
The only values treated as fields are the values passed in the logging call. In the example above, ``field_key=10i`` is a field.
All other values are treated as tags. In the example above, ``msg="log",mod="your_crate::main",ver="1.2.1"`` are tags.

## Performance
The project comes with a benchmark test for the serialization. On the test machine, the serializer is capable of serializing ~1 mio messages per second.
 