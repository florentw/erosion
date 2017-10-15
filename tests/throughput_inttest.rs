extern crate erosion;

use erosion::throughput;

use std::time::Duration;
use std::thread;

fn trigger_event(event_index: &u64) {
    thread::sleep(Duration::new(0, 100));
    if event_index % 100 == 0 {
        println!("Event count {}", event_index);
    }
}

#[test]
fn throughput_generator_calls_event_function_the_right_amount_of_time() {
    let mut event_source = erosion::throughput::EventSource { trigger_event };
    erosion::throughput::single_thread_throughput_interval(1.0, None, event_source);
}