use std::time::{Duration, Instant};
use std::thread;


fn main() {
    let start = Instant::now();

    run_at(1_000);

    println!("Elapsed: {} ms", to_millis(start.elapsed()));
}

const SMALL_INTERVAL_DIVIDER: u32 = 10;
const TOTAL_INTERVAL_MILLIS: u64 = 10_000;
const ROUND_FACTOR: f64 = 100.0;

fn run_at(target_throughput: u64) -> bool {
    let small_interval: Duration = Duration::from_millis(TOTAL_INTERVAL_MILLIS / SMALL_INTERVAL_DIVIDER as u64);
    let zero_duration: Duration = Duration::from_millis(0);

    let mut running = true;
    let mut events_sent: u64 = 0;
    let events_per_small_period: u64 = target_throughput / SMALL_INTERVAL_DIVIDER as u64;
    let mut events_remainder: u64 = target_throughput % SMALL_INTERVAL_DIVIDER as u64;
    let init_remainder: u64 = events_remainder;
    let mut last_sleep = zero_duration;
    let main_target_interval: Duration = Duration::from_millis(TOTAL_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut now;

    for interval in 0..SMALL_INTERVAL_DIVIDER {
        let small_period_start = Instant::now();
        let mut additional_event: u64 = 0;

        if events_remainder > 0 && interval as u64 >= (SMALL_INTERVAL_DIVIDER as u64 - init_remainder) {
            additional_event = 1;
            events_remainder -= 1;
        }

        println!("events_per_small_period: {}, events_remainder: {}, additional_event: {}, main_target_interval(s): {}", events_per_small_period, events_remainder, additional_event, main_target_interval.as_secs());

        for j in 0..(events_per_small_period + additional_event) {
            if !running {
                // is_work_complete() ||
                return true;
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

        // Init last_sleep with first sleep value
        if last_sleep == zero_duration && to_sleep > zero_duration {
            last_sleep = to_sleep * SMALL_INTERVAL_DIVIDER;
        }

        // Sleep avg(last_avg_sleep, current_sleep)
        let avg_sleep: Duration = ((last_sleep / SMALL_INTERVAL_DIVIDER) + to_sleep) / 2;
        println!("avg_sleep: {:?}, last_sleep:{:?}, to_sleep:{:?}", avg_sleep, last_sleep, to_sleep);

        sleep_if_too_fast(avg_sleep);
    }

    let to_sleep = remaining_sleep_duration(start, main_target_interval);
    last_sleep += to_sleep;

    let too_fast = sleep_if_too_fast(to_sleep);

    let total_elapsed = Instant::now().duration_since(start);
    let achieved_throughput: f64 = achieved_throughput(events_sent, total_elapsed);
    let throughput_distance: f64 = round_to_two_places(achieved_throughput - (target_throughput as f64 / 10.0));

    println!("Throughput={} event/s - Was too slow:{} - Distance to target={} events/s - toSleep: {:?}", achieved_throughput, too_fast, throughput_distance, to_sleep);

    return false;
}

fn to_millis(elapsed: Duration) -> u64 {
    return (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
}

fn trigger_event(event_index: &u64) {
    thread::sleep(Duration::new(0, 50));
    if event_index % 100 == 0 {
        println!("Event count {}", event_index);
    }
}

fn remaining_sleep_duration(start: Instant, main_target_interval: Duration) -> Duration {
    let now = Instant::now();

    // Protect against negative durations
    if now > start && main_target_interval > now.duration_since(start) {
        return main_target_interval - now.duration_since(start);
    } else {
        return Duration::from_millis(0);
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
    return round_to_two_places(throughput);
}

fn round_to_two_places(raw_throughput: f64) -> f64 {
    let throughput: f64 = raw_throughput * ROUND_FACTOR;
    return throughput.round() as f64 / ROUND_FACTOR;
}