use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write};

use RetCode::*;

use crate::isa::*;
use crate::machine::{Machine, MachineError};
use crate::machine::MachineStatus::Stopped;
use crate::mem::segs;

const FIRST_FD: i32 = 3;


pub(crate) struct Environment {
    heap_ptr: i32,
    files_open: HashMap<i32, File>,
    next_fd: i32,
}


impl Default for Environment {
    fn default() -> Self {
        Environment {
            heap_ptr: segs::HEAP.start(),
            files_open: Default::default(),
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
        Some(errcode) =>
            m.set_error(MachineError::ProgramExit(errcode)),
        None => {}
    };
    OK as i32
}

fn open(m: &mut Machine) -> i32 {
    if let (Some(buf_ptr), Some(buf_len)) = (pop(m), pop(m)) {
        let path_data = match read_machine_memory(m, buf_ptr, buf_len) {
            Err(code) => return code as i32,
            Ok(data) => data,
        };
        let path = match String::from_utf8(path_data) {
            Err(_) => return UTF8Error as i32,
            Ok(s) => s,
        };
        let file = match OpenOptions::new().write(true).read(true).open(path) {
            Err(_) => return GenericIOError as i32,
            Ok(f) => f,
        };
        let fd = m.env.next_fd;
        m.env.next_fd += 1;
        m.env.files_open.insert(fd, file);
        fd
    } else {
        ArgsInvalid as i32
    }
}

fn write(m: &mut Machine) -> i32 {
    if let (Some(fd), Some(buf_ptr), Some(buf_len)) = (pop(m), pop(m), pop(m)) {
        let data = match read_machine_memory(m, buf_ptr, buf_len) {
            Err(code) => return code as i32,
            Ok(data) => data,
        };
        let result = {
            let mut writer: Box<dyn io::Write> = match fd {
                1 => Box::new(io::stdout()),
                2 => Box::new(io::stderr()),
                fd => match m.env.files_open.get(&fd) {
                    Some(file) => Box::new(file),
                    None => return InvalidFileDescriptor as i32,
                },
            };
            match writer.write(&data) {
                Ok(n) => {
                    writer.flush().unwrap();
                    Ok(n)
                }
                Err(err) => Err(err),
            }
        };
        match result {
            Err(_) => GenericIOError as i32,
            Ok(nwritten) => nwritten as i32,
        }
    } else {
        ArgsInvalid as i32
    }
}

fn read(m: &mut Machine) -> i32 {
    if let (Some(fd), Some(buf_ptr), Some(buf_len)) = (pop(m), pop(m), pop(m)) {
        let mut data = vec![0; buf_len as usize];
        let result = {
            let mut reader: Box<dyn io::Read> = match fd {
                1 => Box::new(io::stdin()),
                2 => return InvalidFileDescriptor as i32,
                fd => match m.env.files_open.get(&fd) {
                    Some(file) => Box::new(file),
                    None => return InvalidFileDescriptor as i32,
                },
            };
            reader.read(&mut data)
        };
        let nread = match result {
            Err(_) => return GenericIOError as i32,
            Ok(n) => n as i32,
        };
        match write_machine_memory(m, buf_ptr, nread, data) {
            Ok(_) => nread,
            Err(code) => return code as i32,
        }
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

fn read_machine_memory(m: &mut Machine, buf_ptr: i32, buf_len: i32) -> Result<Vec<u8>, RetCode> {
    bounds_check(buf_ptr, buf_len)?;
    Ok((buf_ptr..(buf_ptr + buf_len))
        .map(|addr| m.load(addr) as u8)
        .collect())
}

fn write_machine_memory(m: &mut Machine, buf_ptr: i32, buf_len: i32, data: Vec<u8>) -> Result<(), RetCode> {
    bounds_check(buf_ptr, buf_len)?;
    for (addr, val) in (buf_ptr..(buf_ptr + buf_len)).zip(data) {
        m.store(addr, val as i32);
    }
    Ok(())
}

fn bounds_check(buf_ptr: i32, buf_len: i32) -> Result<(), RetCode> {
    if buf_ptr < segs::ADDR_SPACE.start || (buf_ptr + buf_len) >= segs::ADDR_SPACE.end {
        return Err(AddressOutOfBounds);
    }
    Ok(())
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
