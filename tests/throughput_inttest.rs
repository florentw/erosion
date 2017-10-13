extern crate erosion;


#[test]
fn throughput_generator_calls_event_function_the_right_amount_of_time() {
    erosion::throughput::single_thread_throughput_interval(1.0, None);
}