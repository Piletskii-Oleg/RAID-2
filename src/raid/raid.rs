use crate::hamming;
use crate::raid::disks::*;
use crate::raid::{Storage, get_power_of_two};
use std::ops::Range;

pub struct Raid<'a, S: Storage> {
    storage: &'a mut S,
    parity_disks: Vec<Disk>,
    parity_count: usize,
}

impl<'a, S: Storage> Raid<'a, S> {
    pub fn from_data(data: &'a mut S) -> Self {
        let parity_count = hamming::parity_bits_count(data.disk_count());
        let capacity = data.disk_capacity();
        Self {
            parity_disks: vec![Disk::new(capacity); parity_count],
            storage: data,
            parity_count,
        }
    }

    fn encode_single_sequence(&mut self, bits: &[bool]) -> Result<(), String> {
        let bits_extra = hamming::add_parity_bits(bits);
        let parity_bits = hamming::calculate_parity_bits(&bits_extra);

        for (index, bit) in parity_bits.into_iter() {
            self.parity_disks[get_power_of_two(index + 1)].write_bit(bit)?; // not very safe
        }

        Ok(())
    }

    pub fn write_sequence(&mut self, bits: &[bool]) -> Result<(), String> {
        let before_layer = self.storage.last_layer();
        match self.storage.write_sequence(bits) {
            Err(error) => Err(error),
            Ok(()) => {
                let after_layer = self.storage.last_layer();
                for layer in before_layer..after_layer {
                    self.encode_single_sequence(&self.storage.get_data_layer(layer).unwrap())?; // also not safe
                }
                Ok(())
            }
        }
    }

    fn construct_hamming_code(&self, layer: usize) -> Vec<bool> {
        let mut code = Vec::new();
        let mut data_index = 0;
        let mut parity_index = 0;

        while data_index + parity_index != self.parity_count + self.storage.disk_count() {
            if (data_index + parity_index + 1).is_power_of_two() {
                code.push(self.parity_disks[parity_index].get(layer).unwrap());
                parity_index += 1;
            } else {
                code.push(self.storage.get_disk(data_index).get(layer).unwrap());
                data_index += 1;
            }
        }
        code
    }

    pub fn get_slice(&mut self, range: Range<usize>) -> Result<Vec<bool>, String> {
        let starting_layer = self.storage.get_layer_number(range.start);
        let ending_layer = self.storage.get_layer_number(range.end - 1);

        for layer in starting_layer..=ending_layer {
            self.try_fix_error(range.start, layer);
        }

        self.storage.get_slice(range)
    }

    fn try_fix_error(&mut self, start_index: usize, layer: usize) {
        // TODO: только для data-битов
        if let (_, Some(spot)) = hamming::decode(&self.construct_hamming_code(layer)) {
            self.storage.flip_bit_at(start_index + spot + 1);
            if let (_, Some(_)) = hamming::decode(&self.construct_hamming_code(layer)) {
                panic!("no way bro");
            }
        }
    }

    pub fn get_bit(&mut self, index: usize) -> Result<bool, String> {
        self.get_slice(index..index + 1).map(|element| element[0])
    }
}

#[cfg(test)]
mod tests {
    use crate::raid::disks::*;
    use crate::raid::raid::*;

    #[test]
    fn raid_write_test() {
        let mut disks = DiskStorage::new(4, 16);
        let mut raid = Raid::from_data(&mut disks);
        raid.write_sequence(&[
            false, true, false, true, false, true, true, false, true,
        ]).unwrap();

        assert_eq!(raid.parity_disks[0].get(0).unwrap(), false);
        assert_eq!(raid.parity_disks[1].get(0).unwrap(), true);
        assert_eq!(raid.parity_disks[2].get(0).unwrap(), false);

        assert_eq!(
            raid.storage.get_data_layer(0).unwrap(),
            [false, true, false, true]
        );
        assert_eq!(
            raid.storage.get_data_layer(1).unwrap(),
            [false, true, true, false]
        );

        assert_eq!(raid.parity_disks[0].get(1).unwrap(), true);
        assert_eq!(raid.parity_disks[1].get(1).unwrap(), true);
        assert_eq!(raid.parity_disks[2].get(1).unwrap(), false);

        assert_eq!(raid.parity_disks[0].get(2), None);
        assert_eq!(raid.parity_disks[1].get(2), None);
        assert_eq!(raid.parity_disks[2].get(2), None);
    }

    #[test]
    fn raid_construct_hamming_code_test() {
        let mut disks = DiskStorage::new(4, 16);
        let mut raid = Raid::from_data(&mut disks);
        raid.write_sequence(&[
            false, true, false, true, false, true, true, false, true,
        ]).unwrap();

        let code = raid.construct_hamming_code(0);
        assert_eq!(code, [false, true, false, false, true, false, true]);
    }

    #[test]
    fn raid_get_slice_test() {
        let mut disks = DiskStorage::new(4, 16);
        let mut raid = Raid::from_data(&mut disks);

        raid.write_sequence(&[false, false, true, true]).unwrap();
        raid.write_sequence(&[true, true, true, true]).unwrap();

        let slice = raid.get_slice(1..6).unwrap();
        assert_eq!(slice, &[false, true, true, true, true]);

        let slice = raid.get_slice(4..8).unwrap();
        assert_eq!(slice, &[true, true, true, true]);
    }

    #[test]
    fn raid_get_slice_can_fix_error_test() {
        let mut disks = DiskStorage::new(4, 16);
        let mut raid = Raid::from_data(&mut disks);

        raid.write_sequence(&[false, false, true, true]).unwrap();
        raid.write_sequence(&[true, true, true, true]).unwrap();

        // raid.data.disks[0].info[1] = false;
        // let slice = raid.get_slice(1..6).unwrap();
        // assert_eq!(slice, &[false, true, true, true, true]);
        // assert_eq!(raid.data.disks[0].info[1], true);
    }

    #[test]
    fn raid_get_bit_test() {
        let mut disks = DiskStorage::new(4, 16);
        let mut raid = Raid::from_data(&mut disks);

        raid.write_sequence(&[false, false, false, true]).unwrap();
        raid.write_sequence(&[false, true, true, true]).unwrap();

        assert_eq!(raid.get_bit(2).unwrap(), false);
        assert_eq!(raid.get_bit(5).unwrap(), true);
    }
}
