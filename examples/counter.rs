use std::{ops::Deref, process::Command, time::Duration};

use ipc_test::synchronizer::Synchronizer;

fn main() {
    println!("hello world");
    let mut synchronizer = Synchronizer::new("/tmp/count".as_ref());

    let mut count = read(&mut synchronizer);

    // 抢锁
    while count != read(&mut synchronizer) {
        count = read(&mut synchronizer);
    }

    if count < 8 {
        println!("write count: {}", count + 1);
        write(&mut synchronizer, count + 1);

        let output =
            Command::new("/home/isbest/Documents/isbest/ipc_test/target/debug/examples/counter")
                .output()
                .expect("Failed to execute command");
        println!("{}", String::from_utf8_lossy(&output.stdout));

        let output =
            Command::new("/home/isbest/Documents/isbest/ipc_test/target/debug/examples/counter")
                .output()
                .expect("Failed to execute command");

        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

fn read(synchronizer: &mut Synchronizer) -> i32 {
    synchronizer
        .read::<i32>()
        .map(|count| *count.deref())
        .unwrap_or(0)
}

fn write(synchronizer: &mut Synchronizer, count: i32) {
    // 最长等待5s
    synchronizer.write(&count, Duration::from_secs(5)).unwrap();
}
