fn main() {
    let mut data = raid_2::Data::new(5, 1024);
    let mut disks = raid_2::Raid::from_data(&mut data);

    disks
        .write_sequence(&vec![false, false, true, false, false])
        .unwrap();
    let slice = disks.get_slice(0..5).unwrap();

    println!("{:?}", slice);
}
