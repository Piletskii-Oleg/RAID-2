use crate::hamming;

#[derive(Clone)]
enum DiskType {
    Data,
    Parity,
}

#[derive(Clone)]
struct Disk {
    info: Vec<bool>,
    disk_type: DiskType,
}

struct Disks {
    data: Vec<Disk>,
    parity: Vec<Disk>,
    data_disks: usize,
    parity_disks: usize,
    last_index: usize,
}

impl Disk {
    fn new(disk_size: usize, disk_type: DiskType) -> Self {
        Self {
            info: Vec::with_capacity(disk_size),
            disk_type
        }
    }

    fn write(&mut self, bit: bool) {
        self.info.push(bit);
    }

    fn get(&self, index: usize) -> Result<bool, &str> {
        if index > self.info.len() {
            Err("Index was too big.")
        } else {
            Ok(self.info[index])
        }
    }

    fn get_last(&self) -> Result<bool, &str> {
        self.get(self.info.len() - 1)
    }
}

impl Disks {
    fn new(data_disks: usize, disk_size: usize) -> Self {
        let parity_disks = hamming::parity_bits_count(data_disks);
        Self {
            data_disks,
            parity_disks,
            data: vec![Disk::new(disk_size, DiskType::Data); data_disks],
            parity: vec![Disk::new(disk_size, DiskType::Parity); parity_disks],
            last_index: 0,
        }
    }

    fn write_sequence(&mut self, bits: &[bool]) {
        for (index, value) in bits.iter().enumerate() {
            let adjusted_index = index % self.data_disks;
            self.data[adjusted_index].write(*value);
        }
        self.last_index += bits.len();
    }

    fn encode_single_sequence(&mut self, bits: &[bool]) {
        let parity_bits = hamming::calculate_parity_bits(bits);

        for (index, (_, value)) in parity_bits.iter().enumerate() {
            self.parity[index].write(*value);
        }
    }

    fn encode_sequence(&self, bits: &[bool]) {

    }

    fn get_bit(&self, index: usize) -> Result<bool, &str> {
        if index > self.last_index {
            return Err("Index was too big.");
        }

        let disk_number = index % self.data_disks;
        let adjusted_index = index / self.data_disks;
        self.data[disk_number].get(adjusted_index)
    }

    fn get_slice(&self, start_index: usize, end_index: usize) -> Result<Vec<bool>, &str> {
        if end_index > self.last_index {
            return Err("End index is larger than the biggest possible index.");
        }

        let mut result = Vec::with_capacity(end_index - start_index);
        for index in start_index..end_index {
            result.push(self.get_bit(index).unwrap()) // TODO: remove unwrap
        }

        Ok(result)
    }

    fn is_layer_full(&self, layer_index: usize) -> bool {
        layer_index < self.last_index / self.data_disks ||
            (layer_index == self.last_index / self.data_disks && self.last_index % self.data_disks == 0)
    }

    fn get_data_layer(&self, layer_index: usize) -> Result<Vec<bool>, &str> {
        if layer_index > self.last_index / self.data_disks || !self.is_layer_full(layer_index) {
            return Err("Layer is not full");
        }

        let mut layer = Vec::with_capacity(self.data_disks);
        for i in 0..layer.capacity() {
            layer.push(self.data[i].get(layer_index).unwrap());
        }
        Ok(layer)
    }
}

#[cfg(test)]
mod tests {
    use crate::raid::{Disk, Disks, DiskType};

    #[test]
    fn disk_write_get_test() {
        let mut disk = Disk::new(16, DiskType::Data);
        disk.write(false);
        disk.write(true);

        assert_eq!(false, disk.get(0).unwrap());
        assert_eq!(true, disk.get(1).unwrap());
    }

    #[test]
    fn disk_get_last_test() {
        let mut disk = Disk::new(16, DiskType::Data);
        disk.write(false);
        disk.write(true);

        assert_eq!(true, disk.get_last().unwrap());
    }

    #[test]
    fn disks_write_single_sequence_test() {
        let mut disks = Disks::new(4, 16);

        disks.write_sequence(vec![false, false, true, true].as_slice());
        assert_eq!(disks.data[0].get(0).unwrap(), false);
        assert_eq!(disks.data[1].get(0).unwrap(), false);
        assert_eq!(disks.data[2].get(0).unwrap(), true);
        assert_eq!(disks.data[3].get(0).unwrap(), true);
        assert_eq!(disks.last_index, 4);

        disks.write_sequence(vec![true, true, false, true].as_slice());
        assert_eq!(disks.data[0].get(1).unwrap(), true);
        assert_eq!(disks.data[1].get(1).unwrap(), true);
        assert_eq!(disks.data[2].get(1).unwrap(), false);
        assert_eq!(disks.data[3].get(1).unwrap(), true);
        assert_eq!(disks.last_index, 8);
    }

    #[test]
    fn disks_write_multi_layer_sequence_test() {
        let mut disks = Disks::new(4, 16);
        disks.write_sequence(vec![true, false, true, true, false, false].as_slice());
        assert_eq!(disks.data[0].get(0).unwrap(), true);
        assert_eq!(disks.data[1].get(0).unwrap(), false);
        assert_eq!(disks.data[2].get(0).unwrap(), true);
        assert_eq!(disks.data[3].get(0).unwrap(), true);

        assert_eq!(disks.data[0].get(1).unwrap(), false);
        assert_eq!(disks.data[1].get(1).unwrap(), false);
    }

    #[test]
    fn disks_read_slice_test() {
        let mut disks = Disks::new(4, 16);

        disks.write_sequence(vec![false, false, true, true].as_slice());
        disks.write_sequence(vec![true, true, true, true].as_slice());

        let slice = disks.get_slice(1, 6).unwrap();
        assert_eq!(slice, &[false, true, true, true, true])
    }

    #[test]
    fn disks_read_bit_test() {
        let mut disks = Disks::new(4, 16);

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
        let mut disks = Disks::new(4, 16);

        disks.write_sequence(vec![false, true, false, true, false, true, true, false, true].as_slice());

        assert_eq!(disks.get_data_layer(0).unwrap(), [false, true, false, true]);
        assert_eq!(disks.get_data_layer(1).unwrap(), [false, true, true, false]);
        assert_eq!(disks.get_data_layer(2), Err("Layer is not full"));
    }

}