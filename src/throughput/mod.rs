use std::time::{Duration, Instant};
use std::thread;

const SMALL_INTERVAL_DIVIDER: u32 = 10;
const TOTAL_INTERVAL_MILLIS: u64 = 10_000;
const ROUND_FACTOR: f64 = 100.0;

pub struct ThroughputResults {
    // Total sleeping time during throughput iteration
    last_sleep: Duration,

    // Was the throughput iteration interrupted before finishing?
    was_interrupted: bool
}

pub fn single_thread_throughput_interval(target_throughput: f64, previous_results: Option<ThroughputResults>) -> ThroughputResults {
    let start = Instant::now();

    let target_throughput_int: u64 = (target_throughput * SMALL_INTERVAL_DIVIDER as f64).ceil() as u64;
    let small_interval: Duration = Duration::from_millis(TOTAL_INTERVAL_MILLIS / SMALL_INTERVAL_DIVIDER as u64);
    let events_per_small_period: u64 = target_throughput_int / SMALL_INTERVAL_DIVIDER as u64;
    let mut events_remainder: u64 = target_throughput_int % SMALL_INTERVAL_DIVIDER as u64;
    let main_target_interval: Duration = Duration::from_millis(TOTAL_INTERVAL_MILLIS);
    let init_remainder: u64 = events_remainder;

    let mut running = true;
    let mut events_sent: u64 = 0;
    let mut now;

    let mut previous_sleep: Duration;
    if previous_results.is_none() {
        previous_sleep = Duration::from_millis(0);
    } else {
        previous_sleep = previous_results.unwrap().last_sleep;
    }

    for interval in 0..SMALL_INTERVAL_DIVIDER {
        let small_period_start = Instant::now();
        let mut additional_event: u64 = 0;

        if should_add_event_from_remainder(events_remainder, interval, init_remainder) {
            additional_event = 1;
            events_remainder -= 1;
        }

        println!("events_per_small_period: {}, events_remainder: {}, additional_event: {}, main_target_interval(s): {}, previous_sleep: {:?}", //
                 events_per_small_period, events_remainder, additional_event, main_target_interval.as_secs(), previous_sleep);

        for _ in 0..(events_per_small_period + additional_event) {
            if !running {
                // is_work_complete() ||
                return ThroughputResults { last_sleep: previous_sleep, was_interrupted: true };
            }

            trigger_event(&events_sent);
            events_sent += 1;
        }

        now = Instant::now();

        // Exit if finished
        if now.duration_since(start) >= main_target_interval {
            println!("Small interval duration exceeded: {:?}", now.duration_since(start));
            break;
        }

        let to_sleep = remaining_sleep_duration(small_period_start, small_interval);

        if previous_sleep == Duration::from_millis(0) {
            previous_sleep = to_sleep * SMALL_INTERVAL_DIVIDER;
        }

        // Sleep avg(last_avg_sleep, current_sleep)
        let avg_sleep: Duration = ((previous_sleep / SMALL_INTERVAL_DIVIDER) + to_sleep) / 2;
        println!("avg_sleep: {:?}, last_sleep:{:?}, to_sleep:{:?}", avg_sleep, previous_sleep, to_sleep);

        sleep_if_too_fast(avg_sleep);
    }

    let to_sleep = remaining_sleep_duration(start, main_target_interval);
    let too_fast = sleep_if_too_fast(to_sleep);
    previous_sleep += to_sleep;

    let total_elapsed = Instant::now().duration_since(start);
    let achieved_throughput: f64 = achieved_throughput(events_sent, total_elapsed);
    let throughput_distance: f64 = round_to_two_places(achieved_throughput - target_throughput);

    println!("Throughput={} event/s - Was too slow:{} - Distance to target={} events/s - toSleep: {:?}", achieved_throughput, too_fast, throughput_distance, to_sleep);

    return ThroughputResults { last_sleep: previous_sleep, was_interrupted: false };
}

pub fn to_millis(elapsed: Duration) -> u64 {
    (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64
}

fn trigger_event(event_index: &u64) {
    thread::sleep(Duration::new(0, 100));
    if event_index % 100 == 0 {
        println!("Event count {}", event_index);
    }
}

fn remaining_sleep_duration(start: Instant, main_target_interval: Duration) -> Duration {
    let now = Instant::now();

    // Protect against negative durations
    if now > start && main_target_interval > now.duration_since(start) {
        main_target_interval - now.duration_since(start)
    } else {
        Duration::from_millis(0)
    }
}

fn sleep_if_too_fast(to_sleep: Duration) -> bool {
    if to_sleep > Duration::from_millis(0) {
        // too fast
        thread::sleep(to_sleep);
        return true;
    } else {
        // too slow
        return false;
    }
}

fn achieved_throughput(events_sent: u64, elapsed: Duration) -> f64 {
    if to_millis(elapsed) == 0 {
        return -1.0;
    }

    let throughput: f64 = (events_sent as f64 / to_millis(elapsed) as f64) * 1_000.0;

    round_to_two_places(throughput)
}

fn round_to_two_places(raw_throughput: f64) -> f64 {
    let throughput: f64 = raw_throughput * ROUND_FACTOR;
    throughput.round() as f64 / ROUND_FACTOR
}

fn should_add_event_from_remainder(current_events_remainder: u64, current_interval_index: u32, initial_remainder: u64) -> bool {
    current_events_remainder > 0 && current_interval_index as u64 >= (SMALL_INTERVAL_DIVIDER as u64 - initial_remainder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_event_from_remainder_returns_false_when_no_remainder() {
        let has_remainder = should_add_event_from_remainder(0, 2, 10);

        assert_eq!(false, has_remainder);
    }

    #[test]
    fn should_add_event_from_remainder_returns_true_when_index_is_in_remainder_interval() {
        let has_remainder = should_add_event_from_remainder(2, 2, 10);

        assert_eq!(true, has_remainder);
    }

    #[test]
    fn should_add_event_from_remainder_returns_false_when_index_is_outside_remainder_interval() {
        let has_remainder = should_add_event_from_remainder(2, 2, 1);

        assert_eq!(false, has_remainder);
    }

    #[test]
    fn round_function_rounds_to_two_places() {
        let rounded = round_to_two_places(10.009);

        assert_eq!(10.01, rounded);
    }

    #[test]
    fn sleep_if_too_fast_returns_true_if_duration_not_zero() {
        let too_fast = sleep_if_too_fast(Duration::new(0, 1));

        assert_eq!(true, too_fast);
    }

    #[test]
    fn sleep_if_too_fast_returns_false_if_duration_zero() {
        let too_fast = sleep_if_too_fast(Duration::new(0, 0));

        assert_eq!(false, too_fast);
    }

    #[test]
    fn achieved_throughput_returns_rounded_throughput() {
        let throughput = achieved_throughput(1337, Duration::new(9, 0));

        assert_eq!(148.56, throughput);
    }

    #[test]
    fn achieved_throughput_returns_minus_one_when_elapsed_is_zero() {
        let throughput = achieved_throughput(1, Duration::new(0, 0));

        assert_eq!(-1.0, throughput);
    }

    #[test]
    fn remaining_sleep_duration_returns_zero_when_start_is_in_the_future() {
        use std::ops::Add;
        let start = Instant::now();

        let sleep_duration = remaining_sleep_duration(start.add(Duration::from_millis(1000)), Duration::from_millis(10_000));

        assert_eq!(Duration::from_millis(0), sleep_duration);
    }

    #[test]
    fn to_millis_converts_duration_to_integer() {
        let millis = to_millis(Duration::from_millis(1337));

        assert_eq!(1337, millis);
    }
}
