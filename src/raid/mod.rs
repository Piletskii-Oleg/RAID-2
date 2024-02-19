pub mod raid;

pub mod disks;

use std::ops::Range;

pub trait Storage {
    fn disk_count(&self) -> usize;
    fn disk_capacity(&self) -> usize;
    fn write_sequence(&mut self, bits: &[bool]) -> Result<(), String>;
    fn get_bit(&self, index: usize) -> Option<bool>;
    fn get_slice(&self, range: Range<usize>) -> Result<Vec<bool>, String>;
    fn last_layer(&self) -> usize;
    fn get_data_layer(&self, layer_index: usize) -> Result<Vec<bool>, String>;
    fn get_layer_number(&self, index: usize) -> usize;
    fn get_disk(&self, index: usize) -> &disks::Disk;
    fn flip_bit_at(&mut self, index: usize);
}

fn get_power_of_two(num: usize) -> usize {
    let mut result = num;
    let mut count = 0;
    while result > 1 {
        result >>= 1;
        count += 1;
    }
    count
}
