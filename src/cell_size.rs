
use std::{iter};

pub trait CellSize {
    fn get_zeroes(count: usize) -> iter::Take<iter::Repeat<Self>> 
    where Self: Sized, 
          iter::Repeat<Self>: Iterator;

    fn add_to_cell(&mut self, value: Self);
    fn sub_from_cell(&mut self, value: Self);

    fn is_zero(&self) -> bool;
    fn is_nonzero(&self) -> bool;

    fn from_stdout(c: u8) -> Self where Self: Sized;
    fn to_stdin(&self) -> u8 where Self: Sized;

    fn from_tk_value(v: i32) -> Self where Self: Sized;
}

impl CellSize for u8 {
    fn get_zeroes(count: usize) -> iter::Take<iter::Repeat<u8>> {
        iter::repeat(0).take(count)
    }

    fn add_to_cell(&mut self, value: u8) {
        *self = self.wrapping_add(value)
    }

    fn sub_from_cell(&mut self, value: u8) {
        *self = self.wrapping_sub(value)
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }

    fn is_nonzero(&self) -> bool {
        *self > 0
    }

    fn from_stdout(c: u8) -> u8 { c }
    fn to_stdin(&self) -> u8 { *self }

    fn from_tk_value(v: i32) -> u8 { v as u8 }
}

impl CellSize for u16 {
    fn get_zeroes(count: usize) -> iter::Take<iter::Repeat<u16>> {
        iter::repeat(0).take(count)
    }

    fn add_to_cell(&mut self, value: u16) {
        *self = self.wrapping_add(value)
    }

    fn sub_from_cell(&mut self, value: u16) {
        *self = self.wrapping_sub(value)
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }

    fn is_nonzero(&self) -> bool {
        *self > 0
    }

    fn from_stdout(c: u8) -> u16 { c as u16 }
    fn to_stdin(&self) -> u8 { *self as u8 }

    fn from_tk_value(v: i32) -> u16 { v as u16 }
}

impl CellSize for u32 {
    fn get_zeroes(count: usize) -> iter::Take<iter::Repeat<u32>> {
        iter::repeat(0).take(count)
    }

    fn add_to_cell(&mut self, value: u32) {
        *self = self.wrapping_add(value)
    }

    fn sub_from_cell(&mut self, value: u32) {
        *self = self.wrapping_sub(value)
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }

    fn is_nonzero(&self) -> bool {
        *self > 0
    }

    fn from_stdout(c: u8) -> u32 { c as u32 }
    fn to_stdin(&self) -> u8 { *self as u8 }

    fn from_tk_value(v: i32) -> u32 { v as u32 }
}

impl CellSize for u64 {
    fn get_zeroes(count: usize) -> iter::Take<iter::Repeat<u64>> {
        iter::repeat(0).take(count)
    }

    fn add_to_cell(&mut self, value: u64) {
        *self = self.wrapping_add(value)
    }

    fn sub_from_cell(&mut self, value: u64) {
        *self = self.wrapping_sub(value)
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }

    fn is_nonzero(&self) -> bool {
        *self > 0
    }

    fn from_stdout(c: u8) -> u64 { c as u64 }
    fn to_stdin(&self) -> u8 { *self as u8 }

    fn from_tk_value(v: i32) -> u64 { v as u64 }
}
