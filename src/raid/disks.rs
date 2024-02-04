use std::ops::Range;

#[derive(Clone)]
pub(crate) struct Disk {
    pub(super) info: Vec<bool>,
}

pub struct Data {
    pub(super) disks: Vec<Disk>,
    pub(super) disk_count: usize,
    pub(super) last_index: usize,
    pub(super) last_layer: usize,
    pub(super) total_capacity: usize,
}

struct FileInfo {
    //data, file_type, name
}

struct File {
    //start_pos, end_pos, file_info
}

impl Disk {
    pub fn new(disk_size: usize) -> Self {
        Self {
            info: Vec::with_capacity(disk_size),
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        self.info.push(bit);
    }

    fn flip_at(&mut self, index: usize) {
        self.info[index] ^= true;
    }

    pub fn get(&self, index: usize) -> Result<bool, &str> {
        if index >= self.info.len() {
            Err("Index was too big.")
        } else {
            Ok(self.info[index])
        }
    }

    fn get_last(&self) -> Result<bool, &str> {
        self.get(self.info.len() - 1)
    }
}

impl Data {
    pub fn new(disk_count: usize, disk_size: usize) -> Self {
        Self {
            disk_count,
            disks: vec![Disk::new(disk_size); disk_count],
            last_index: 0,
            last_layer: 0,
            total_capacity: disk_count * disk_size,
        }
    }

    pub fn write_sequence(&mut self, bits: &[bool]) -> Result<(), &str> {
        if self.last_index + bits.len() >= self.total_capacity {
            return Err("Not enough space");
        }

        let previous_last_index = self.last_index;
        for (index, &bit) in bits.iter().enumerate() {
            let adjusted_index = (previous_last_index + index) % self.disk_count;
            self.disks[adjusted_index].write_bit(bit);
            if adjusted_index == self.disk_count - 1 {
                self.last_layer += 1;
            }
            self.last_index += 1;
        }
        Ok(())
    }

    pub fn get_bit(&self, index: usize) -> Result<bool, &str> {
        if index > self.last_index {
            return Err("Index was too big.");
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

    pub fn get_slice(&self, range: Range<usize>) -> Result<Vec<bool>, &str> {
        if range.end > self.last_index {
            return Err("End index is larger than the biggest possible index.");
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

    pub(super) fn get_data_layer(&self, layer_index: usize) -> Result<Vec<bool>, &str> {
        if layer_index > self.last_index / self.disk_count || !self.is_layer_full(layer_index) {
            return Err("Layer is not full");
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

#[cfg(test)]
mod tests {
    use crate::raid::disks::{Data, Disk};

    #[test]
    fn disk_write_get_test() {
        let mut disk = Disk::new(16);
        disk.write_bit(false);
        disk.write_bit(true);

        assert_eq!(false, disk.get(0).unwrap());
        assert_eq!(true, disk.get(1).unwrap());
    }

    #[test]
    fn disk_get_last_test() {
        let mut disk = Disk::new(16);
        disk.write_bit(false);
        disk.write_bit(true);

        assert_eq!(true, disk.get_last().unwrap());
    }

    #[test]
    fn disks_write_single_sequence_test() {
        let mut disks = Data::new(4, 16);

        disks.write_sequence(vec![false, false, true, true].as_slice());
        assert_eq!(disks.disks[0].get(0).unwrap(), false);
        assert_eq!(disks.disks[1].get(0).unwrap(), false);
        assert_eq!(disks.disks[2].get(0).unwrap(), true);
        assert_eq!(disks.disks[3].get(0).unwrap(), true);
        assert_eq!(disks.last_index, 4);

        disks.write_sequence(vec![true, true, false, true].as_slice());
        assert_eq!(disks.disks[0].get(1).unwrap(), true);
        assert_eq!(disks.disks[1].get(1).unwrap(), true);
        assert_eq!(disks.disks[2].get(1).unwrap(), false);
        assert_eq!(disks.disks[3].get(1).unwrap(), true);
        assert_eq!(disks.last_index, 8);
    }

    #[test]
    fn disks_write_multi_layer_sequence_test() {
        let mut disks = Data::new(4, 16);
        disks.write_sequence(vec![true, false, true, true, false, false].as_slice());
        assert_eq!(disks.disks[0].get(0).unwrap(), true);
        assert_eq!(disks.disks[1].get(0).unwrap(), false);
        assert_eq!(disks.disks[2].get(0).unwrap(), true);
        assert_eq!(disks.disks[3].get(0).unwrap(), true);

        assert_eq!(disks.disks[0].get(1).unwrap(), false);
        assert_eq!(disks.disks[1].get(1).unwrap(), false);

        disks.write_sequence(vec![true, false, true].as_slice());
        assert_eq!(disks.disks[2].get(1).unwrap(), true);
        assert_eq!(disks.disks[3].get(1).unwrap(), false);
        assert_eq!(disks.disks[0].get(2).unwrap(), true);
    }

    #[test]
    fn disks_read_slice_test() {
        let mut disks = Data::new(4, 16);

        disks.write_sequence(vec![false, false, true, true].as_slice());
        disks.write_sequence(vec![true, true, true, true].as_slice());

        let slice = disks.get_slice((1..6)).unwrap();
        assert_eq!(slice, &[false, true, true, true, true])
    }

    #[test]
    fn disks_read_bit_test() {
        let mut disks = Data::new(4, 16);

        disks.write_sequence(vec![false, true, false, true].as_slice());
        disks.write_sequence(vec![false, true, true, false].as_slice());

        assert_eq!(disks.get_bit(3).unwrap(), true);
        assert_eq!(disks.get_bit(4).unwrap(), false);
        assert_eq!(disks.get_bit(5).unwrap(), true);
        assert_eq!(disks.get_bit(6).unwrap(), true);
        assert_eq!(disks.get_bit(7).unwrap(), false);
    }

    #[test]
    fn disks_get_layer_test() {
        let mut disks = Data::new(4, 16);

        disks
            .write_sequence(
                vec![false, true, false, true, false, true, true, false, true].as_slice(),
            )
            .unwrap();

        assert_eq!(disks.get_data_layer(0).unwrap(), [false, true, false, true]);
        assert_eq!(disks.get_data_layer(1).unwrap(), [false, true, true, false]);
        assert_eq!(disks.get_data_layer(2), Err("Layer is not full"));
    }
}
