use std::io;
use std::cell::RefCell;
use slog::{FnValue, PushFnValue};
use slog::{OwnedKVList, KV, SendSyncRefUnwindSafeKV};
use slog::{Record, o};
use crate::{Client, Error};
use crate::ser::{TelegrafSocketSerializer};

pub struct TelegrafDrain {
    values: Vec<OwnedKVList>,
    client: RefCell<Client>,
    measurement: String
}

impl TelegrafDrain {
    pub fn new(url: String, measurement: String) -> Result<TelegrafDrain, Error>  {
        Ok(TelegrafDrainBuilder::new(Client::new(url)?, measurement).with_default_keys().build())
    }
}

impl slog::Drain for TelegrafDrain  {
    type Ok = ();
    type Err = io::Error;

    fn log(&self, rinfo: &Record, logger_values: &OwnedKVList) -> io::Result<()> {
        let mut serializer = TelegrafSocketSerializer::start(&self.measurement, None)?;

        for kv in &self.values {
            kv.serialize(rinfo, &mut serializer)?;
        }

        // NOTE: The logger values get serialized as tags
        // If you want to change this behavior, move this line below serializer.tag_value_break()
        // and before serializer.end()
        logger_values.serialize(rinfo, &mut serializer)?;

        serializer.tag_value_break()?;

        rinfo.kv().serialize(rinfo, &mut serializer)?;

        let data = serializer.end()?;
        self.client.borrow_mut().write(data.as_bytes())
    }
}

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

    pub fn build(self) -> TelegrafDrain {
        TelegrafDrain {
            values: self.values,
            client: RefCell::new(self.client),
            measurement: self.measurement
        }
    }
    pub fn add_key_value<T>(mut self, value: slog::OwnedKV<T>) -> Self
        where T: SendSyncRefUnwindSafeKV + 'static
    {
        self.values.push(value.into());
        self
    }

    pub fn with_default_keys(self) -> Self {
        self.add_key_value(o!(
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
    fn test_logging() {
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
        thread::sleep(std::time::Duration::from_micros(500));

        let drain = TelegrafDrain::new("tcp://127.0.0.1:63742".into(), "test".into()).unwrap().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = Logger::root(drain, o!("ver" => "1.2.1"));
        info!(log, "log"; "testy" => 10);

        let sent_message = receiver.recv().unwrap().unwrap();
        assert_eq!(sent_message, "test,mod=\"slog_telegraf::drain::test\",msg=\"log\",level=4i,ver=\"1.2.1\" testy=10i\n");
    }
}