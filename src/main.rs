use raid_2::Raid;

fn main() {
    let mut disks = raid_2::DiskStorage::new(5, 1024);
    let mut data = Raid::from_data(&mut disks);

    data
        .write_sequence(&vec![false, false, true, false, false])
        .unwrap();
    let slice = data.get_slice(0..5).unwrap();

    println!("{:?}", slice);
}
