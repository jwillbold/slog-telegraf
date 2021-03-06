[![Build Status](https://travis-ci.com/jwillbold/slog-telegraf.svg?token=hPh87VpFt3MQPwdySdkS&branch=master)](https://travis-ci.com/jwillbold/slog-telegraf)
[![codecov](https://codecov.io/gh/jwillbold/slog-telegraf/branch/master/graph/badge.svg?token=2EQLM7NCG1)](https://codecov.io/gh/jwillbold/slog-telegraf)
[![crates.io](https://img.shields.io/crates/v/slog-telegraf.svg)](https://crates.io/crates/slog-telegraf)
[![docs.rs](https://img.shields.io/badge/docs.rs-current-blue.svg)](https://docs.rs/slog-telegraf)

# slog-telegraf

[Telegraf](https://www.influxdata.com/time-series-platform/telegraf/) drain for [slog-rs](https://github.com/slog-rs/slog).
Formats the log message and sends it using TCP or UDP to Telegraf. 

Feel free to open issues or pull requests.

## Usage

The logger supports the [TCP and UDP socket listener](https://github.com/influxdata/telegraf/blob/release-1.14/plugins/inputs/socket_listener/README.md) 
of Telegraf and serializes messages according to the [line protocol](https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_tutorial/#syntax).

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
    let drain = TelegrafDrain::new("tcp://127.0.0.1:8094".into(), "measurement".into()).unwrap().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let log = Logger::root(drain, o!("ver" => "1.2.1"));
    info!(log, "log"; "field_key" => 10);
}
```

## Notes
The only values treated as fields are the values passed in the logging call. In the example above, ``field_key=10i`` is a field.
All other values are treated as tags. In the example above, ``msg=log,mod=your_crate::main,ver=1.2.1`` are tags. Since tags my not contain
whitespaces, it is up to the user to ensure that tag values contain no whitespaces or commas.

Further, neither tag nor fields keys may contain whitespaces. Thus, log messages as well as all slog value keys may not contain whitespaces.
slog-telegraf will not validate messages. Instead they will be filtered by Telegraf and not appear in the database.

Since messages with no fields (messages with not parameter besides the log message) are not considered valid by InfluxDB, slog-telegraf appends the dummy
field '_dummy=1i' to otherwise field-less messages.

## Performance
The project comes with a benchmark test for the serialization. On the test machine, the serializer is capable of serializing ~1 mio messages per second.

If you care more about performance and less about every log message actually arriving, which is also the design philosophy of slog, 
it is recommended to use the UDP socket.
 