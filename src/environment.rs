use std::io;
use std::io::{Read, Write};

use RetCode::*;

use crate::isa::*;
use crate::machine::{Machine, MachineError};
use crate::machine::MachineStatus::Stopped;
use crate::mem::segs;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};

const FIRST_FD: i32 = 3;


pub(crate) struct Environment {
    heap_ptr: i32,
    open_files: HashMap<i32, File>,
    next_fd: i32,
}


impl Default for Environment {
    fn default() -> Self {
        Environment {
            heap_ptr: segs::HEAP.start(),
            open_files: Default::default(),
            next_fd: FIRST_FD,
        }
    }
}

pub enum RetCode {
    UTF8Error = -5,
    GenericIOError = -4,
    InvalidFileDescriptor = -3,
    AddressOutOfBounds = -2,
    ArgsInvalid = -1,
    OK = 0,
}

fn exit(m: &mut Machine) -> i32 {
    match pop(m) {
        Some(0) =>
            m.set_status(Stopped),
        Some(status) =>
            m.set_error(MachineError::ProgramExit(status)),
        None => {},
    };
    OK as i32
}

fn open(m: &mut Machine) -> i32 {
    if let (Some(path_buf), Some(buf_len)) = (pop(m), pop(m)) {
        let data = match read_data_from_machine(m, path_buf, buf_len) {
            Err(code) => return code,
            Ok(data) => data,
        };
        let path = match String::from_utf8(data) {
            Err(_) => return UTF8Error as i32,
            Ok(s) => s,
        };
        let file = match OpenOptions::new().write(true).read(true).open(path) {
            Err(_) => return GenericIOError as i32,
            Ok(f) => f,
        };
        let fd = m.env.next_fd;
        m.env.next_fd += 1;
        m.env.open_files.insert(fd, file);
        fd
    } else {
        ArgsInvalid as i32
    }
}

fn write(m: &mut Machine) -> i32 {
    if let (Some(fd), Some(buf), Some(buf_len)) = (pop(m), pop(m), pop(m)) {
        let data = match read_data_from_machine(m, buf, buf_len) {
            Err(code) => return code as i32,
            Ok(data) => data,
        };
        let mut writer: Box<dyn io::Write> = match fd {
            1 => Box::new(io::stdout()),
            2 => Box::new(io::stderr()),
            fd => match m.env.open_files.get(&fd) {
                None => return InvalidFileDescriptor as i32,
                Some(file) => Box::new(file),
            },
        };
        let result = writer.write(&data);
        writer.flush().unwrap();
        match result {
            Err(_) => GenericIOError as i32,
            Ok(n) => n as i32,
        }
    } else {
        ArgsInvalid as i32
    }
}

fn read_data_from_machine(m: &mut Machine, buf: i32, buf_len: i32) -> Result<Vec<u8>, RetCode> {
    if !bounds_check(buf, buf_len) {
        return Err(AddressOutOfBounds);
    }
    Ok((buf..(buf + buf_len))
        .map(|addr| m.unsafe_load(addr) as u8)
        .collect())
}

fn read(m: &mut Machine) -> i32 {
    if let (Some(fd), Some(buf), Some(buf_len)) = (pop(m), pop(m), pop(m)) {
        if let Err(code) = bounds_check(buf, buf_len) {
            return code as i32;
        }
        let mut data = vec![0; buf_len as usize];
        let result = {
            let mut reader: Box<dyn io::Read> = match fd {
                1 => Box::new(io::stdin()),
                fd => match m.env.open_files.get(&fd) {
                    None => return InvalidFileDescriptor as i32,
                    Some(file) => Box::new(file),
                },
            };
            reader.read(&mut data)
        };
        let nread = match result {
            Err(_) => return GenericIOError as i32,
            Ok(n) => n as i32,
        };
        for (addr, val) in (buf..(buf + buf_len)).zip(data) {
            m.unsafe_store(addr, val as i32);
        }
        nread
    } else {
        ArgsInvalid as i32
    }
}

fn malloc(m: &mut Machine) -> i32 {
    if let Some(size) = pop(m) {
        if m.env.heap_ptr + size >= segs::HEAP.end() {
            return 0; // out of memory
        }
        let ptr = m.env.heap_ptr;
        m.env.heap_ptr += size;
        ptr
    } else {
        ArgsInvalid as i32
    }
}

fn bounds_check(buf: i32, buf_len: i32) -> bool {
    buf >= segs::ADDR_SPACE.start && (buf + buf_len) < segs::ADDR_SPACE.end
}

macro_rules! def_env_call_list {
    ( $($name:ident)+ ) => {
        pub const CALL_LIST: &[(fn(&mut Machine) -> i32, &'static str)] = &[
            $(
                ($name, stringify!($name)),
            )+
        ];
    }
}

def_env_call_list![
    exit
    open
    write
    read
    malloc
];
