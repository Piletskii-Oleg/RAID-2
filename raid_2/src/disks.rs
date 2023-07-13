use std::ops::Range;
use crate::hamming;

#[derive(Clone)]
struct Disk {
    info: Vec<bool>,
}

struct Data {
    disks: Vec<Disk>,
    disk_count: usize,
    last_index: usize,
    last_layer: usize,
    total_capacity: usize
}

struct Raid<'a> {
    data: &'a mut Data,
    parity_disks: Vec<Disk>,
    parity_count: usize
}

fn get_power_of_two(num: usize) -> usize {
    let mut result = num;
    let mut count = 0;
    while result > 1 {
        result = result >> 1;
        count += 1;
    }
    count
}

impl Disk {
    fn new(disk_size: usize) -> Self {
        Self {
            info: Vec::with_capacity(disk_size),
        }
    }

    fn write(&mut self, bit: bool) {
        self.info.push(bit);
    }

    fn flip_at(&mut self, index: usize) {
        self.info[index] ^= true;
    }

    fn get(&self, index: usize) -> Result<bool, &str> {
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
    fn new(disk_count: usize, disk_size: usize) -> Self {
        Self {
            disk_count,
            disks: vec![Disk::new(disk_size); disk_count],
            last_index: 0,
            last_layer: 0,
            total_capacity: disk_count * disk_size,
        }
    }

    fn write_sequence(&mut self, bits: &[bool]) -> Result<(), &str> {
        if self.last_index + bits.len() >= self.total_capacity {
            return Err("Not enough space");
        }

        let previous_last_index = self.last_index;
        for (index, value) in bits.iter().enumerate() {
            let adjusted_index = (previous_last_index + index) % self.disk_count;
            self.disks[adjusted_index].write(*value);
            if adjusted_index == self.disk_count - 1 {
                self.last_layer += 1;
            }
            self.last_index += 1;
        }
        Ok(())
    }

    fn get_bit(&self, index: usize) -> Result<bool, &str> {
        if index > self.last_index {
            return Err("Index was too big.");
        }

        let disk_number = index % self.disk_count;
        let adjusted_index = index / self.disk_count;
        self.disks[disk_number].get(adjusted_index)
    }

    fn flip_bit_at(&mut self, index: usize) {
        let disk_number = index % self.disk_count;
        let adjusted_index = index / self.disk_count;
        self.disks[disk_number].flip_at(adjusted_index);
    }

    fn get_slice(&self, range: Range<usize>) -> Result<Vec<bool>, &str> {
        if range.end > self.last_index {
            return Err("End index is larger than the biggest possible index.");
        }

        let mut result = Vec::with_capacity(range.len());
        for index in range {
            result.push(self.get_bit(index).unwrap()) // TODO: remove unwrap
        }

        Ok(result)
    }

    fn is_layer_full(&self, layer_index: usize) -> bool {
        layer_index < self.last_index / self.disk_count ||
            (layer_index == self.last_index / self.disk_count && self.last_index % self.disk_count == 0)
    }

    fn get_data_layer(&self, layer_index: usize) -> Result<Vec<bool>, &str> {
        if layer_index > self.last_index / self.disk_count || !self.is_layer_full(layer_index) {
            return Err("Layer is not full");
        }

        let mut layer = Vec::with_capacity(self.disk_count);
        for i in 0..layer.capacity() {
            layer.push(self.disks[i].get(layer_index).unwrap());
        }
        Ok(layer)
    }

    fn get_layer_number(&self, index: usize) -> usize {
        index / self.disk_count
    }
}

fn is_power_of_two(num: usize) -> bool {
    num.count_ones() == 1 || num == 0
}

impl<'a> Raid<'a> {
    pub fn new(data: &'a mut Data) -> Self {
        let parity_count = hamming::parity_bits_count(data.disk_count);
        Self {
            parity_disks: vec![Disk::new(data.disks[0].info.capacity()); parity_count],
            data,
            parity_count,
        }
    }

    fn encode_single_sequence(&mut self, bits: &[bool]) {
        let bits_extra = hamming::add_bits(bits);
        let parity_bits = hamming::calculate_parity_bits(&bits_extra);

        for (index, value) in parity_bits.into_iter() {
            self.parity_disks[get_power_of_two(index + 1)].write(value);
        }
    }

    pub fn write_sequence(&mut self, bits: &[bool]) -> Result<(), String> {
        let before_layer = self.data.last_layer;
        match self.data.write_sequence(bits) {
            Err(error) => Err(error.to_owned()),
            Ok(()) => {
                let after_layer = self.data.last_layer;
                for layer in before_layer..after_layer {
                    self.encode_single_sequence(self.data.get_data_layer(layer).unwrap().as_slice());
                }
                Ok(())
            }
        }
    }

    fn construct_hamming_code(&self, layer: usize) -> Vec<bool> {
        let mut code = Vec::new();
        let mut data_index = 0;
        let mut parity_index = 0;
        while data_index + parity_index != self.parity_count + self.data.disk_count {
            if is_power_of_two(data_index + parity_index + 1) {
                code.push(self.parity_disks[parity_index].get(layer).unwrap());
                parity_index += 1;
            } else {
                code.push(self.data.disks[data_index].get(layer).unwrap());
                data_index += 1;
            }
        }
        code
    }

    pub fn get_slice(&mut self, range: Range<usize>) -> Result<Vec<bool>, &str> {
        let starting_layer = self.data.get_layer_number(range.start);
        let ending_layer = self.data.get_layer_number(range.end);

        for layer in starting_layer..=ending_layer {
            self.try_fix_error(range.start, layer);
        }

        self.data.get_slice(range)
    }

    fn try_fix_error(&mut self, start_index: usize, layer: usize) {
        if let (_, Some(spot)) = hamming::hamming_decode(&self.construct_hamming_code(layer)) {
            self.data.flip_bit_at(start_index + spot + 1);
            if let (_, Some(_)) = hamming::hamming_decode(&self.construct_hamming_code(layer)) {
                panic!("no way bro");
            }
        }
    }

    fn get_bit(&mut self, index: usize) -> Result<bool, &str> {
        match self.get_slice((index..index + 1)) {
            Ok(element) => Ok(element[0]),
            Err(error) => Err(error),
        }
    }


}

#[cfg(test)]
mod tests {
    use crate::disks::{Disk, Data, Raid};

    #[test]
    fn disk_write_get_test() {
        let mut disk = Disk::new(16);
        disk.write(false);
        disk.write(true);

        assert_eq!(false, disk.get(0).unwrap());
        assert_eq!(true, disk.get(1).unwrap());
    }

    #[test]
    fn disk_get_last_test() {
        let mut disk = Disk::new(16);
        disk.write(false);
        disk.write(true);

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

        disks.write_sequence(vec![false, true, false, true, false, true, true, false, true].as_slice());

        assert_eq!(disks.get_data_layer(0).unwrap(), [false, true, false, true]);
        assert_eq!(disks.get_data_layer(1).unwrap(), [false, true, true, false]);
        assert_eq!(disks.get_data_layer(2), Err("Layer is not full"));
    }

    #[test]
    fn raid_write_test() {
        let mut disks = Data::new(4, 16);
        let mut raid = Raid::new(&mut disks);
        raid.write_sequence(vec![false, true, false, true, false, true, true, false, true].as_slice());

        assert_eq!(raid.parity_disks[0].get(0).unwrap(), false);
        assert_eq!(raid.parity_disks[1].get(0).unwrap(), true);
        assert_eq!(raid.parity_disks[2].get(0).unwrap(), false);

        assert_eq!(raid.data.get_data_layer(0).unwrap(), [false, true, false, true]);
        assert_eq!(raid.data.get_data_layer(1).unwrap(), [false, true, true, false]);

        assert_eq!(raid.parity_disks[0].get(1).unwrap(), true);
        assert_eq!(raid.parity_disks[1].get(1).unwrap(), true);
        assert_eq!(raid.parity_disks[2].get(1).unwrap(), false);

        assert_eq!(raid.parity_disks[0].get(2), Err("Index was too big."));
        assert_eq!(raid.parity_disks[1].get(2), Err("Index was too big."));
        assert_eq!(raid.parity_disks[2].get(2), Err("Index was too big."));
    }

    #[test]
    fn raid_construct_hamming_code_test() {
        let mut disks = Data::new(4, 16);
        let mut raid = Raid::new(&mut disks);
        raid.write_sequence(vec![false, true, false, true, false, true, true, false, true].as_slice());

        let code = raid.construct_hamming_code(0);
        assert_eq!(code, [false, true, false, false, true, false, true]);
    }

    #[test]
    fn raid_get_slice_test() {
        let mut disks = Data::new(4, 16);
        let mut raid = Raid::new(&mut disks);

        raid.write_sequence(vec![false, false, true, true].as_slice());
        raid.write_sequence(vec![true, true, true, true].as_slice());

        let slice = raid.get_slice((1..6)).unwrap();
        assert_eq!(slice, &[false, true, true, true, true])
    }

    #[test]
    fn raid_get_slice_can_fix_error_test() {
        let mut disks = Data::new(4, 16);
        let mut raid = Raid::new(&mut disks);

        raid.write_sequence(vec![false, false, true, true].as_slice());
        raid.write_sequence(vec![true, true, true, true].as_slice());

        raid.data.disks[0].info[1] = false;
        let slice = raid.get_slice((1..6)).unwrap();
        assert_eq!(slice, &[false, true, true, true, true]);
        assert_eq!(raid.data.disks[0].info[1], true);
    }

    #[test]
    fn raid_get_bit_test() {
        let mut disks = Data::new(4, 16);
        let mut raid = Raid::new(&mut disks);

        raid.write_sequence(vec![false, false, false, true].as_slice());
        raid.write_sequence(vec![false, true, true, true].as_slice());

        assert_eq!(raid.get_bit(2).unwrap(), false);
        assert_eq!(raid.get_bit(5).unwrap(), true);
    }
}