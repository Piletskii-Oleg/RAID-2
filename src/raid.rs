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
            info: vec![false; disk_size],
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

    fn write_single_sequence(&mut self, bits: &[bool]) {
        for (index, value) in bits.iter().enumerate() {
            self.data[index].write(*value);
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn write_test() {

    }
}