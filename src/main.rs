use raid_2::Raid;

fn main() {
    let mut data = raid_2::DiskStorage::new(5, 1024);
    let disks = Box::new(data);
    let mut disks = Raid::from_data(disks);

    disks
        .write_sequence(&vec![false, false, true, false, false])
        .unwrap();
    let slice = disks.get_slice(0..5).unwrap();

    println!("{:?}", slice);
}
