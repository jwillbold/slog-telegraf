use std::io;
use std::cell::RefCell;
use slog::{FnValue, PushFnValue};
use slog::{OwnedKVList, KV, SendSyncRefUnwindSafeKV};
use slog::{Record, o};
use crate::{Client, Error};
use crate::ser::{TelegrafSocketSerializer};


/// Telegraf `Drain` for `slog-rs`.
///
/// ``` no_run
///use slog::{Logger, Drain, o, info};
///use slog_telegraf::{TelegrafDrain};
///
///fn main() {
///    let drain = TelegrafDrain::new("tcp://127.0.0.1:8094".into(), "measurement".into()).unwrap().fuse();
///    let drain = slog_async::Async::new(drain).build().fuse();
///
///    let log = Logger::root(drain, o!("ver" => "1.2.1"));
///    info!(log, "log"; "field_key" => 10);
///}
/// ```
pub struct TelegrafDrain {
    values: Vec<OwnedKVList>,
    client: RefCell<Client>,
    measurement: String
}

impl TelegrafDrain {
    pub fn new(url: String, measurement: String) -> Result<TelegrafDrain, Error>  {
        Ok(TelegrafDrainBuilder::new(Client::new(url)?, measurement).default_tags().build())
    }
}

impl slog::Drain for TelegrafDrain  {
    type Ok = ();
    type Err = io::Error;

    fn log(&self, rinfo: &Record, logger_values: &OwnedKVList) -> io::Result<()> {
        let mut serializer = TelegrafSocketSerializer::start(&self.measurement, None)?;
        let mut tag_serializer = serializer.tag_serializer();

        for kv in &self.values {
            kv.serialize(rinfo, &mut tag_serializer)?;
        }

        // NOTE: The logger values get serialized as tags
        // If you want to change this behavior, move this line below serializer.tag_value_break()
        // and before serializer.end()
        logger_values.serialize(rinfo, &mut tag_serializer)?;
        serializer.tag_value_break()?;

        let mut field_serializer = serializer.field_serializer();
        rinfo.kv().serialize(rinfo, &mut field_serializer)?;

        let data = serializer.end()?;
        self.client.borrow_mut().write(data.as_bytes())
    }
}

/// Telegraf `Drain` builder
///
/// ```no_run
/// use slog::*;
/// use slog_telegraf::{TelegrafDrainBuilder, Client};
///
/// let client = Client::new("tcp://127.0.0.1:8094".into()).unwrap();
///
/// let drain = TelegrafDrainBuilder::new(client, "measurement".into())
///                 .add_tag_kv(o!("key" => "value")).build().fuse();
/// // ...
/// ```
pub struct TelegrafDrainBuilder {
    values: Vec<OwnedKVList>,
    client: Client,
    measurement: String
}

impl TelegrafDrainBuilder {
    pub fn new(client: Client, measurement: String) -> Self {
        TelegrafDrainBuilder {
            values: vec![],
            client,
            measurement
        }
    }

    /// Build the 'Drain'
    pub fn build(self) -> TelegrafDrain {
        TelegrafDrain {
            values: self.values,
            client: RefCell::new(self.client),
            measurement: self.measurement
        }
    }

    /// Add custom tags to be used in every log statement
    pub fn add_tag_kv<T>(mut self, value: slog::OwnedKV<T>) -> Self
        where T: SendSyncRefUnwindSafeKV + 'static
    {
        self.values.push(value.into());
        self
    }

    /// Adds default tags
    ///
    /// * `level` - record logging level integer, "Critical is the smallest and Trace the biggest value" - slog::Level, docs.rs/slog
    /// * `msg` - The logged message
    /// * `mod` - The source module of the log message, e.g. 'your_crate::main'
    pub fn default_tags(self) -> Self {
        self.add_tag_kv(o!(
            "level" => FnValue(move |rinfo| rinfo.level().as_usize()),
            "msg" => PushFnValue(move |record, ser| ser.emit(record.msg())),
            "mod" => FnValue(move |rinfo| rinfo.module()),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{thread, net};
    use std::sync::mpsc;
    use slog::{Logger, Drain};

    #[test]
    fn test_tcp_logging() {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || sender.send(|| -> std::io::Result<String> {
            use std::io::{BufReader, BufRead};

            let listener = net::TcpListener::bind("127.0.0.1:63742")?;
            println!("Listening...");

            let stream = listener.incoming().next().unwrap()?;
            println!("Incoming...");

            let mut line = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut line).unwrap();

            Ok(line)
        }()));

        // Wait fot the listener
        thread::sleep(std::time::Duration::from_millis(300));

        let drain = TelegrafDrain::new("tcp://127.0.0.1:63742".into(), "test".into()).unwrap().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = Logger::root(drain, o!("ver" => "1.2.1"));
        info!(log, "log"; "testy" => 10);

        let recvd_message = receiver.recv().unwrap().unwrap();
        assert_eq!(recvd_message, "test,mod=slog_telegraf::drain::test,msg=log,level=4,ver=1.2.1 testy=10i\n");
    }

    #[test]
    fn test_udp_logging() {
        let socket = net::UdpSocket::bind("127.0.0.1:63743").unwrap();

        let drain = TelegrafDrain::new("udp://127.0.0.1:63743".into(), "test".into()).unwrap().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = Logger::root(drain, o!("ver" => "1.2.1"));

        // This should ensure that at least one message arrives
        for _ in 0..10 {
            info!(log, "log"; "testy" => 10);
        }

        let mut buf = [0u8; 4096];

        socket.recv(&mut buf).unwrap();
        let recvd_message =std::str::from_utf8(&buf).unwrap().trim_matches(char::from(0));

        assert_eq!(recvd_message, "test,mod=slog_telegraf::drain::test,msg=log,level=4,ver=1.2.1 testy=10i\n");
    }
}
