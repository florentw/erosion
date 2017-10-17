extern crate erosion;

use std::time::Duration;
use std::thread;
use std::sync::{Mutex, Arc};

#[test]
fn throughput_generator_calls_event_function_once_a_sec() {
    let results = assert_throughput_is_met(10.0, Duration::from_millis(90));

    assert_eq!(0.0, results.throughput_distance);
    assert!(results.last_sleep > Duration::new(0, 500_000_000));
}

#[test]
fn throughput_generator_calls_event_function_hundred_times_a_sec() {
    let results = assert_throughput_is_met(100.0, Duration::new(0, 100_000));

    assert!(results.throughput_distance < 0.09);
    assert!(results.last_sleep > Duration::new(9, 700_000_000));
}

#[test]
fn throughput_generator_calls_event_function_three_hundred_times_a_sec() {
    let results = assert_throughput_is_met(300.0, Duration::new(0, 30_000));

    assert!(results.throughput_distance < 0.5);
    assert!(results.last_sleep > Duration::new(9, 700_000_000));
}

#[test]
fn throughput_generator_calls_event_function_five_hundred_times_a_sec() {
    let results = assert_throughput_is_met(500.0, Duration::new(0, 10_000));

    assert!(results.throughput_distance < 0.9);
    assert!(results.last_sleep > Duration::new(9, 500_000_000));
}

#[test]
fn throughput_generator_calls_event_function_thousands_times_a_sec() {
    let results = assert_throughput_is_met(1_000.0, Duration::new(0, 0));

    assert!(results.throughput_distance < 0.7);
    assert!(results.last_sleep > Duration::new(9, 800_000_000));
}

struct TestEventSource {
    pub event_elapsed: Duration,
    pub counter: Arc<Mutex<u64>>
}

impl erosion::throughput::EventSource for TestEventSource {
    fn trigger_event(&self, _event_index: &u64) {
        let local_counter = self.counter.clone();
        let mut num = local_counter.lock().unwrap();
        *num += 1;

        if self.event_elapsed > Duration::from_millis(0) {
            thread::sleep(self.event_elapsed);
        }
    }
}

const ITERATIONS: u64 = 10;

fn assert_throughput_is_met(target_throughput: f64, event_elapsed: Duration) -> erosion::throughput::ThroughputResults {
    let counter = Arc::new(Mutex::new(0));
    let local_counter = counter.clone();
    let event_source = TestEventSource { counter, event_elapsed };

    let throughput_results = erosion::throughput::single_thread_throughput_interval(event_source, target_throughput, None);

    assert!(!throughput_results.is_none());
    let results = throughput_results.unwrap();
    println!("throughput_results: last sleep: {:?}, distance: {:?}", results.last_sleep, results.throughput_distance);

    assert_eq!(ITERATIONS * target_throughput as u64, *local_counter.lock().unwrap());
    results
}
