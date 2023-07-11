use std::collections::HashMap;
use std::ops::BitXor;

fn main() {
    let vec = vec![1,0,0,1,1,0,1,0];
    let vec = num_to_bool(&vec);
    let a = hamming_encode(&vec);
    println!("{:?}", a);
}

fn num_to_bool(values: &Vec<i32>) -> Vec<bool> {
    let mut bits = vec![];
    for value in values.into_iter() {
        if *value == 0 {
            bits.push(false);
        } else {
            bits.push(true);
        }
    }
    bits
}

fn hamming_encode(bits: &Vec<bool>) -> Vec<bool> {
    let mut encoded = add_bits(bits);

    let controlling_bits = calculate_controlling_bits(&encoded, encoded.len() - bits.len());

    for (index, value) in controlling_bits.into_iter() {
        encoded[index] = value;
    }

    encoded
}

fn add_bits(bits: &Vec<bool>) -> Vec<bool> {
    let mut encoded = Vec::new();
    let mut index = 0;
    let mut total_index = 0;
    while index != bits.len() {
        if is_power_of_two(total_index + 1) {
            encoded.push(false);
        } else {
            encoded.push(bits[index]);
            index += 1;
        }
        total_index += 1;
    }
    encoded
}

fn calculate_controlling_bits(bits: &Vec<bool>, bits_amount: usize) -> HashMap<usize, bool>{
    let mut controlling_bits = HashMap::with_capacity(bits_amount);
    for (index, _) in bits.iter().enumerate() {
        if is_power_of_two(index + 1) {
            controlling_bits.insert(index, calculate_bit_at(bits, index + 1));
        }
    }
    controlling_bits
}

fn calculate_bit_at(bits: &Vec<bool>, position: usize) -> bool {
    let mut bit = false;
    let shift = position.ilog2();
    for (index, value) in bits.into_iter().enumerate() {
        if bit_at_position(index + 1, shift) == 1 {
            bit = bit.bitxor(value);
        }
    }
    bit
}

fn bit_at_position(number: usize, shift: u32) -> usize {
    ((number) >> (shift)) % 2
}

fn is_power_of_two(num: usize) -> bool {
    num.count_ones() == 1 || num == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_at_position_test() { // 22 = 0b10110
        assert_eq!(0, bit_at_position(22, 0));
        assert_eq!(1, bit_at_position(22, 1));
        assert_eq!(1, bit_at_position(22, 2));
        assert_eq!(0, bit_at_position(22, 3));
        assert_eq!(1, bit_at_position(22, 4));
    }

    #[test]
    fn is_power_of_two_test() {
        assert_eq!(true, is_power_of_two(1));
        assert_eq!(true, is_power_of_two(4));
        assert_eq!(false, is_power_of_two(545));
        assert_eq!(true, is_power_of_two(1024));
    }

    #[test]
    fn hamming_encode_test() {
        let vec = num_to_bool(&vec![1,0,0,1,1,0,1,0]);
        let encoded = hamming_encode(&vec);
        assert_eq!(vec![false, true, true, true, false, false, true, false, true, false, true, false], encoded);
    }
}