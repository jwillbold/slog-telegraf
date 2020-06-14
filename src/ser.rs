use std::fmt;
use std::fmt::Write;
use slog::{Key};

#[doc(hidden)]
// TelegrafSocketSerializer is only exported to use it in benchmarks. It is not considered
// stable API.
pub struct TelegrafSocketSerializer {
    data: String,
    skip_comma: bool
}

impl TelegrafSocketSerializer {
    pub fn start(measurement: &str, len: Option<usize>) -> Result<Self, slog::Error> {
        let mut data = String::with_capacity(len.unwrap_or(120));
        data.write_str(measurement)?;

        Ok(TelegrafSocketSerializer { data, skip_comma: false})
    }

    pub fn tag_value_break(&mut self) -> slog::Result {
        self.skip_comma = true;
        self.data.write_char(' ').map_err(|e| e.into())
    }

    pub fn end(self) -> Result<String, slog::Error> {
        let mut data = self.data;
        data.write_char('\n')?;
        Ok(data)
    }

    fn maybe_write_comma(&mut self) -> slog::Result {
        if self.skip_comma {
            self.skip_comma = false;
        } else {
            self.data.write_char(',')?;
        }

        Ok(())
    }

    fn write_int(&mut self, key: Key, integer: i64) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!("{}={}i", key, integer)).map_err(|e| e.into())
    }

    fn write_float(&mut self, key: Key, float: f64) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!("{}={}", key, float)).map_err(|e| e.into())
    }
}

impl slog::Serializer for TelegrafSocketSerializer {
    fn emit_u8(&mut self, key: Key, val: u8) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_i8(&mut self, key: Key, val: i8) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_u16(&mut self, key: Key, val: u16) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_i16(&mut self, key: Key, val: i16) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_usize(&mut self, key: Key, val: usize) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_isize(&mut self, key: Key, val: isize) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_u32(&mut self, key: Key, val: u32) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_i32(&mut self, key: Key, val: i32) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_u64(&mut self, key: Key, val: u64) -> slog::Result {
        self.write_int(key, val as i64)
    }

    fn emit_i64(&mut self, key: Key, val: i64) -> slog::Result {
        self.write_int(key, val)
    }


    fn emit_f32(&mut self, key: Key, val: f32) -> slog::Result {
        self.write_float(key, val as f64)
    }

    fn emit_f64(&mut self, key: Key, val: f64) -> slog::Result {
        self.write_float(key, val)
    }


    fn emit_bool(&mut self, key: Key, val: bool) -> slog::Result {
        self.maybe_write_comma()?;
        if val {
            self.data.write_fmt(format_args!("{}=t", key)).map_err(|e| e.into())
        } else {
            self.data.write_fmt(format_args!("{}=f", key)).map_err(|e| e.into())
        }
    }


    fn emit_char(&mut self, key: Key, val: char) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!(r#"{}="{}""#, key, val)).map_err(|e| e.into())
    }

    fn emit_str(&mut self, key: Key, val: &str) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!(r#"{}="{}""#, key, val)).map_err(|e| e.into())
    }


    // Serialize '()' as '0'
    fn emit_unit(&mut self, key: Key) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!("{}=0", key)).map_err(|e| e.into())
    }

    // Serialize 'None' as 'false'
    fn emit_none(&mut self, key: Key) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!("{}=f", key)).map_err(|e| e.into())
    }


    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments) -> slog::Result {
        self.maybe_write_comma()?;
        self.data.write_fmt(format_args!("{}=\"{}\"", key, val)).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use slog::{KV, Record, o};

    #[test]
    fn test_serializer() {
        let mut serializer = TelegrafSocketSerializer::start("test_measurement", None).unwrap();

        // rinfo_static and the values passed to Record::new are irrelevant for this test and
        // exist only to fulfill the function arguments
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

        &o!(
            "int0" => 10 as u8,
            "int1" => -10 as i8,
            "int2" => 10000 as u16,
            "int3" => -10000 as i16,
            "int4" => 2_000_000_000 as u32,
            "int5" => -2_000_000_000 as i32,
            "int6" => 2_000_000_000 as usize,
            "int7" => -2_000_000_000 as isize,
            "int8" => 2_000_000_000_000 as u64,
            "int9" => -2_000_000_000_000 as i64,
            "float0" => 13.2 as f32,
            "float1" => -105.2 as f64,
            "string0" => "foo",
            "string1" => "1.2.1",
            "char0" => 'x',
            "bool0" => true,
            "bool1" => false,
            "unit" => (),
            "none" => Option::<()>::None
       ).serialize(&Record::new(&rinfo_static,
                                 &format_args!("msg_{}", "foo"),
                                 slog::BorrowedKV(&o!("key" => "val"))),
                    &mut serializer).unwrap();

        let data = serializer.end().unwrap();

        assert_eq!(data, "test_measurement,none=f,unit=0,bool1=f,bool0=t,char0=\"x\",string1=\"1.2.1\",string0=\"foo\",float1=-105.2,float0=13.199999809265137,int9=-2000000000000i,int8=2000000000000i,int7=-2000000000i,int6=2000000000i,int5=-2000000000i,int4=2000000000i,int3=-10000i,int2=10000i,int1=-10i,int0=10i\n");
    }
}