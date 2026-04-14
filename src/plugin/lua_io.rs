use mlua::{Lua, UserData, UserDataMethods, Result, Error};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::path::Path;
use aho_corasick::AhoCorasick;

#[derive(Clone)]
struct OpenFile(Arc<Mutex<Option<File>>>);

impl UserData for OpenFile {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // read n bytes (if n == 0 -> read_all)
        methods.add_method("read", |_, this, n: Option<usize>| {
            let mut guard = this.0.lock().unwrap();
            let f = guard.as_mut().ok_or_else(|| Error::external("file closed"))?;
            if let Some(n) = n {
                let mut buf = vec![0u8; n];
                let read = f.read(&mut buf).map_err(Error::external)?;
                buf.truncate(read);
                let s = String::from_utf8(buf).map_err(Error::external)?;
                Ok(s)
            } else {
                let mut s = String::new();
                f.read_to_string(&mut s).map_err(Error::external)?;
                Ok(s)
            }
        });

        // read_to_end returns bytes as Lua string
        methods.add_method("read_to_end", |_, this, _: ()| {
            let mut guard = this.0.lock().unwrap();
            let f = guard.as_mut().ok_or_else(|| Error::external("file closed"))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(Error::external)?;
            let s = String::from_utf8(buf).map_err(Error::external)?;
            Ok(s)
        });

        // read_line: read a single line (like BufRead::read_line)
        methods.add_method("read_line", |_, this, _: ()| {
            let mut guard = this.0.lock().unwrap();
            let f = guard.as_mut().ok_or_else(|| Error::external("file closed"))?;
            // Use a temporary BufReader starting at current position
            let mut reader = BufReader::new(f.try_clone().map_err(Error::external)?);
            let mut line = String::new();
            reader.read_line(&mut line).map_err(Error::external)?;
            // advance the original file by the number of bytes read
            let pos = reader.seek(SeekFrom::Current(0)).map_err(Error::external)?;
            f.seek(SeekFrom::Start(pos)).map_err(Error::external)?;
            Ok(line)
        });

        // seek: set position
        methods.add_method("seek", |_, this, pos: u64| {
            let mut guard = this.0.lock().unwrap();
            let f = guard.as_mut().ok_or_else(|| Error::external("file closed"))?;
            f.seek(SeekFrom::Start(pos)).map_err(Error::external)?;
            Ok(())
        });

        // tell: get current position
        methods.add_method("tell", |_, this, _: ()| {
            let mut guard = this.0.lock().unwrap();
            let f = guard.as_mut().ok_or_else(|| Error::external("file closed"))?;
            let p = f.seek(SeekFrom::Current(0)).map_err(Error::external)?;
            Ok(p)
        });

        // close: drop the file
        methods.add_method("close", |_, this, _: ()| {
            let mut guard = this.0.lock().unwrap();
            *guard = None;
            Ok(())
        });
    }
}

fn open_file<P: AsRef<Path>>(path: P) -> Result<OpenFile> {
    let f = File::open(path).map_err(Error::external)?;
    Ok(OpenFile(Arc::new(Mutex::new(Some(f)))))
}
