pub mod raid;

pub mod disks;

fn get_power_of_two(num: usize) -> usize {
    let mut result = num;
    let mut count = 0;
    while result > 1 {
        result >>= 1;
        count += 1;
    }
    count
}
