use std::ops::Range;
use crate::raid::Storage;

#[derive(Clone)]
pub struct Disk {
    pub info: Vec<bool>,
    pub capacity: usize,
}

pub struct DiskStorage {
    pub(super) disks: Vec<Disk>,
    pub(super) disk_count: usize,
    pub(super) last_index: usize,
    pub(super) last_layer: usize,
    pub(super) disk_capacity: usize,
    pub(super) total_capacity: usize,
}

struct FileInfo {
    //data, file_type, name
}

struct File {
    //start_pos, end_pos, file_info
}

impl Disk {
    pub fn new(capacity: usize) -> Self {
        Self {
            info: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<(), String> {
        if self.info.len() >= self.capacity {
            return Err("Disk size limit reached.".to_string());
        }

        self.info.push(bit);
        Ok(())
    }

    fn flip_at(&mut self, index: usize) {
        self.info[index] ^= true;
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        self.info.get(index).copied()
    }

    fn get_last(&self) -> Option<bool> {
        self.get(self.info.len() - 1)
    }
}

impl DiskStorage {
    pub fn new(disk_count: usize, disk_size: usize) -> Self {
        Self {
            disk_count,
            disks: vec![Disk::new(disk_size); disk_count],
            last_index: 0,
            last_layer: 0,
            disk_capacity: disk_size,
            total_capacity: disk_count * disk_size,
        }
    }

    pub fn write_sequence(&mut self, bits: &[bool]) -> Result<(), String> {
        if self.last_index + bits.len() >= self.total_capacity {
            return Err("Not enough space".to_string());
        }

        let previous_last_index = self.last_index;
        for (index, &bit) in bits.iter().enumerate() {
            let adjusted_index = (previous_last_index + index) % self.disk_count;
            self.disks[adjusted_index].write_bit(bit)?;
            if adjusted_index == self.disk_count - 1 {
                self.last_layer += 1;
            }
            self.last_index += 1;
        }
        Ok(())
    }

    pub fn get_bit(&self, index: usize) -> Option<bool> {
        if index > self.last_index {
            return None;
        }

        let disk_number = index % self.disk_count;
        let adjusted_index = index / self.disk_count;
        self.disks[disk_number].get(adjusted_index)
    }

    pub(super) fn flip_bit_at(&mut self, index: usize) {
        let disk_number = index % self.disk_count;
        let adjusted_index = index / self.disk_count;
        self.disks[disk_number].flip_at(adjusted_index);
    }

    pub fn get_slice(&self, range: Range<usize>) -> Result<Vec<bool>, String> {
        if range.end > self.last_index {
            return Err("End index is larger than the biggest possible index.".to_string());
        }

        let mut result = Vec::with_capacity(range.len());
        for index in range {
            result.push(self.get_bit(index).unwrap()) // TODO: remove unwrap
        }

        Ok(result)
    }

    pub(super) fn is_layer_full(&self, layer_index: usize) -> bool {
        layer_index < self.last_index / self.disk_count
            || (layer_index == self.last_index / self.disk_count
                && self.last_index % self.disk_count == 0)
    }

    pub(super) fn get_data_layer(&self, layer_index: usize) -> Result<Vec<bool>, String> {
        if layer_index > self.last_index / self.disk_count || !self.is_layer_full(layer_index) {
            return Err("Layer is not full".to_string());
        }

        let mut layer = Vec::with_capacity(self.disk_count);
        for i in 0..layer.capacity() {
            layer.push(self.disks[i].get(layer_index).unwrap());
        }
        Ok(layer)
    }

    pub(super) fn get_layer_number(&self, index: usize) -> usize {
        index / self.disk_count
    }
}

impl Storage for DiskStorage {
    fn disk_count(&self) -> usize {
        self.disk_count
    }

    fn disk_capacity(&self) -> usize {
        self.disk_capacity
    }

    fn write_sequence(&mut self, bits: &[bool]) -> Result<(), String> {
        self.write_sequence(bits)
    }

    fn get_bit(&self, index: usize) -> Option<bool> {
        self.get_bit(index)
    }

    fn get_slice(&self, range: Range<usize>) -> Result<Vec<bool>, String> {
        self.get_slice(range)
    }

    fn last_layer(&self) -> usize {
        self.last_layer
    }

    fn get_data_layer(&self, layer_index: usize) -> Result<Vec<bool>, String> {
        self.get_data_layer(layer_index)
    }

    fn get_layer_number(&self, index: usize) -> usize {
        self.get_layer_number(index)
    }

    fn get_disk(&self, index: usize) -> &Disk {
        &self.disks[index]
    }

    fn flip_bit_at(&mut self, index: usize) {
        self.flip_bit_at(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::raid::disks::{Disk, DiskStorage};

    #[test]
    fn disk_write_get_test() {
        let mut disk = Disk::new(16);
        disk.write_bit(false).unwrap();
        disk.write_bit(true).unwrap();

        assert_eq!(false, disk.get(0).unwrap());
        assert_eq!(true, disk.get(1).unwrap());
    }

    #[test]
    fn disk_get_last_test() {
        let mut disk = Disk::new(16);
        disk.write_bit(false).unwrap();
        disk.write_bit(true).unwrap();

        assert_eq!(true, disk.get_last().unwrap());
    }

    #[test]
    fn disks_write_single_sequence_test() {
        let mut disks = DiskStorage::new(4, 16);

        disks.write_sequence(&[false, false, true, true]).unwrap();
        assert_eq!(disks.disks[0].get(0).unwrap(), false);
        assert_eq!(disks.disks[1].get(0).unwrap(), false);
        assert_eq!(disks.disks[2].get(0).unwrap(), true);
        assert_eq!(disks.disks[3].get(0).unwrap(), true);
        assert_eq!(disks.last_index, 4);

        disks.write_sequence(&[true, true, false, true]).unwrap();
        assert_eq!(disks.disks[0].get(1).unwrap(), true);
        assert_eq!(disks.disks[1].get(1).unwrap(), true);
        assert_eq!(disks.disks[2].get(1).unwrap(), false);
        assert_eq!(disks.disks[3].get(1).unwrap(), true);
        assert_eq!(disks.last_index, 8);
    }

    #[test]
    fn disks_write_multi_layer_sequence_test() {
        let mut disks = DiskStorage::new(4, 16);
        disks.write_sequence(&[true, false, true, true, false, false]).unwrap();
        assert_eq!(disks.disks[0].get(0).unwrap(), true);
        assert_eq!(disks.disks[1].get(0).unwrap(), false);
        assert_eq!(disks.disks[2].get(0).unwrap(), true);
        assert_eq!(disks.disks[3].get(0).unwrap(), true);

        assert_eq!(disks.disks[0].get(1).unwrap(), false);
        assert_eq!(disks.disks[1].get(1).unwrap(), false);

        disks.write_sequence(&[true, false, true]).unwrap();
        assert_eq!(disks.disks[2].get(1).unwrap(), true);
        assert_eq!(disks.disks[3].get(1).unwrap(), false);
        assert_eq!(disks.disks[0].get(2).unwrap(), true);
    }

    #[test]
    fn disks_read_slice_test() {
        let mut disks = DiskStorage::new(4, 16);

        disks.write_sequence(&[false, false, true, true]).unwrap();
        disks.write_sequence(&[true, true, true, true]).unwrap();

        let slice = disks.get_slice(1..6).unwrap();
        assert_eq!(slice, &[false, true, true, true, true])
    }

    #[test]
    fn disks_read_bit_test() {
        let mut disks = DiskStorage::new(4, 16);

        disks.write_sequence(&[false, true, false, true]).unwrap();
        disks.write_sequence(&[false, true, true, false]).unwrap();

        assert_eq!(disks.get_bit(3).unwrap(), true);
        assert_eq!(disks.get_bit(4).unwrap(), false);
        assert_eq!(disks.get_bit(5).unwrap(), true);
        assert_eq!(disks.get_bit(6).unwrap(), true);
        assert_eq!(disks.get_bit(7).unwrap(), false);
    }

    #[test]
    fn disks_get_layer_test() {
        let mut disks = DiskStorage::new(4, 16);

        disks
            .write_sequence(&[
                false, true, false, true, false, true, true, false, true,
            ])
            .unwrap();

        assert_eq!(disks.get_data_layer(0).unwrap(), [false, true, false, true]);
        assert_eq!(disks.get_data_layer(1).unwrap(), [false, true, true, false]);
        assert_eq!(
            disks.get_data_layer(2),
            Err("Layer is not full".to_string())
        );
    }
}
