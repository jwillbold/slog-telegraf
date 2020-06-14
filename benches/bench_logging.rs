use criterion::{black_box, criterion_group, criterion_main, Criterion};
use slog_telegraf::{TelegrafSocketSerializer};
use slog::{KV, Record, o, OwnedKV};

fn serialize<T: slog::KV+Send+Sync+std::panic::RefUnwindSafe>(kv: &OwnedKV<T>) -> String {
    let mut serializer = TelegrafSocketSerializer::start("test_measurement", None).unwrap();
    let rinfo_static = slog::RecordStatic {
        location: &slog::RecordLocation {
            file: "file",
            line: 0,
            column: 0,
            function: "function",
            module: "slog_telegraf::ser::test"
        },
        tag: "slog_tag",
        level: slog::Level::Info
    };

    kv.serialize(&Record::new(&rinfo_static,
                                 &format_args!("msg_{}", "foo"),
                                 slog::BorrowedKV(&o!("key" => "val"))),
                    &mut serializer).unwrap();

    serializer.end().unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("serialize int", |b| b.iter(|| serialize(black_box(&o!(
            "int0" => 10,
            "int1" => 10000,
            "int2" => -100000123,
            "int3" => 0,
            "int4" => 5_000_000_000 as i64,
            "float0" => 13.2,
            "float1" => -105.2,
            "string0" => "foo",
            "string1" => "1.2.1",
            "string2" => "bar",
            "string3" => "LOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOONG",
            "char0" => 'x',
            "char1" => '!',
            "char2" => '0',
            "char3" => '_',
            "bool0" => true,
            "bool1" => false,
    )))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);