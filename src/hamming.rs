use std::collections::HashMap;

pub fn num_to_bool(values: &[i32]) -> Vec<bool> {
    values.iter().map(|x| *x == 1).collect()
}

pub fn encode(bits: &[bool]) -> Vec<bool> {
    let mut encoded = add_parity_bits(bits);

    let parity_bits = calculate_parity_bits(&encoded);

    for (index, bit) in parity_bits {
        encoded[index] = bit;
    }

    encoded
}

pub fn decode(bits: &[bool]) -> (Vec<bool>, Option<usize>) {
    let parity_bits = calculate_parity_bits(bits);
    let mut error_spot = None;
    for (index, bit) in parity_bits {
        if bit {
            match error_spot {
                None => error_spot = Some(index + 1),
                Some(spot) => error_spot = Some(spot + index + 1),
            }
        }
    }

    if let Some(spot) = error_spot {
        let spot = spot - 1;
        let mut corrected = bits.to_owned();
        corrected[spot] ^= true;
        (remove_parity_bits(&corrected), Some(spot))
    } else {
        (remove_parity_bits(bits), None)
    }
}

pub fn parity_bits_count(len: usize) -> usize {
    let mut count = 0;
    let mut two_power = 1;
    let mut power = 0;
    while two_power < len + power + 1 {
        count += 1;
        two_power *= 2;
        power += 1;
    }

    count
}

pub fn add_parity_bits(bits: &[bool]) -> Vec<bool> {
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

fn remove_parity_bits(bits: &[bool]) -> Vec<bool> {
    let mut decoded = Vec::new();
    for (index, _) in bits.iter().enumerate() {
        if !is_power_of_two(index + 1) {
            decoded.push(bits[index]);
        }
    }
    decoded
}

pub fn calculate_parity_bits(bits: &[bool]) -> HashMap<usize, bool> {
    let mut parity_bits = HashMap::with_capacity(parity_bits_count(bits.len()));
    for (index, _) in bits.iter().enumerate() {
        if is_power_of_two(index + 1) {
            parity_bits.insert(index, calculate_bit_at(bits, index + 1));
        }
    }
    parity_bits
}

fn calculate_bit_at(bits: &[bool], position: usize) -> bool {
    let mut bit = false;
    let shift = position.ilog2();
    for (index, value) in bits.iter().enumerate() {
        if bit_from_right(index + 1, shift) == 1 {
            bit ^= *value;
        }
    }
    bit
}

fn bit_from_right(number: usize, shift: u32) -> usize {
    ((number) >> (shift)) % 2
}

fn is_power_of_two(num: usize) -> bool {
    num.count_ones() == 1 || num == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_at_position_test() {
        // 22 = 0b10110
        assert_eq!(0, bit_from_right(22, 0));
        assert_eq!(1, bit_from_right(22, 1));
        assert_eq!(1, bit_from_right(22, 2));
        assert_eq!(0, bit_from_right(22, 3));
        assert_eq!(1, bit_from_right(22, 4));
    }

    #[test]
    fn hamming_encode_test() {
        let vec = num_to_bool(&vec![1, 0, 0, 1, 1, 0, 1, 0]);
        let encoded = encode(&vec);
        assert_eq!(
            vec![false, true, true, true, false, false, true, false, true, false, true, false],
            encoded
        );
    }

    #[test]
    fn hamming_decode_on_correct_test() {
        let vec = num_to_bool(&vec![1, 0, 0, 1, 1, 0, 1, 0]);
        let encoded = encode(&vec);
        let decoded = decode(&encoded);
        assert_eq!(vec, decoded.0);
    }

    #[test]
    fn hamming_decode_on_incorrect_test() {
        let initial = num_to_bool(&vec![1, 0, 0, 1, 1, 0, 1, 0]);
        let incorrect = num_to_bool(&vec![0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0]); // error on 2nd position
        assert_eq!(decode(&incorrect), (initial, Some(2)));
    }
}
