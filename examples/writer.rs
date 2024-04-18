use ipc_test::synchronizer::Synchronizer;
use std::time::Duration;

fn main() {
    // Initialize the Synchronizer
    let mut synchronizer = Synchronizer::new("/tmp/count".as_ref());

    // Define the data
    let mut data = 1;
    loop {
        println!("writing data: {}", data);
        // Write data to shared memory
        let (written, reset) = synchronizer
            .write(&data, Duration::from_secs(100))
            .expect("failed to write data");

        // Show how many bytes written and whether state was reset
        println!(
            "written: {} bytes | reset: {} | data: {}",
            written, reset, data
        );
        data += 1;
    }
}
