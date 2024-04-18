use std::{ops::Deref, thread, time::Duration};

use ipc_test::synchronizer::Synchronizer;

fn main() {
    // Initialize the Synchronizer
    let mut synchronizer = Synchronizer::new("/tmp/count".as_ref());

    loop {
        // Read data from shared memory
        let data = synchronizer.read::<i32>().expect("failed to read data");

        thread::sleep(Duration::from_secs(5));
        // Access fields of the struct
        println!("data: {}", data.deref());
    }
}
