mod hamming;
mod disks;

fn main() {
    let vec = vec![1,0,0,1,1,0,1,0];
    let vec = hamming::num_to_bool(&vec);
    let a = hamming::hamming_encode(&vec);
    println!("{:?}", a);
}