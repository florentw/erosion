use std::time::Instant;

mod throughput;

fn main() {
    let start = Instant::now();

    let mut results = None;
    for _ in 0..3 {
        results = Option::Some(throughput::single_thread_throughput_interval(100.0, results));
    }

    println!("Elapsed: {} ms", throughput::to_millis(start.elapsed()));
}
