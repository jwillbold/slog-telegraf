use criterion::{black_box, criterion_group, criterion_main, Criterion};
use slog_telegraf::{TelegrafSocketSerializer};
use slog::{KV, Record, o, OwnedKV};

fn serialize<T: slog::SendSyncRefUnwindSafeKV>(kv: &OwnedKV<T>) -> String {
    let mut serializer = TelegrafSocketSerializer::start("test_measurement", None).unwrap();
    let mut tag_serializer = serializer.tag_serializer();

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
                    &mut tag_serializer).unwrap();

    serializer.tag_value_break().unwrap();
    let mut field_serializer = serializer.field_serializer();

    kv.serialize(&Record::new(&rinfo_static,
                              &format_args!("msg_{}", "foo"),
                              slog::BorrowedKV(&o!("key" => "val"))),
                 &mut field_serializer).unwrap();

    let insert_dummy_field = field_serializer.skip_comma;
    serializer.end(insert_dummy_field).unwrap()
}

fn benchmark_serialize(c: &mut Criterion) {
    c.bench_function("serialize int", |b| b.iter(|| serialize(black_box(&o!(
            "int0" => 0,
            "int1" => 10000,
            "int2" => -100000123,
            "int4" => 5_000_000_000 as i64,
            "float0" => 13.2,
            "string0" => "foo",
            "string1" => "1.2.1",
            "char0" => 'x',
            "bool0" => true,
    )))));
}

criterion_group!(benches, benchmark_serialize);
criterion_main!(benches);