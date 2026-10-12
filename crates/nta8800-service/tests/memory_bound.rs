//! Memory of a request: a counting allocator measures what a parsed body
//! costs per JSON value and checks that running an operation does not copy
//! the body (the HTTP memory bound in `docs/nta8800-api.md` counts one
//! parsed body per admitted request).

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use nta8800_service::{
    http::{json_value_count, DEFAULT_MAX_JSON_VALUES},
    operations::{execute, operation},
};
use serde_json::{json, Value};

struct Counting;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() {
            let now = CURRENT.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
            PEAK.fetch_max(now, Ordering::SeqCst);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
        CURRENT.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The tests measure one at a time: the counters are process-wide.
static SERIAL: Mutex<()> = Mutex::new(());

/// Bytes allocated above the start while `run` runs, at its peak.
fn peak_during<R>(run: impl FnOnce() -> R) -> (R, usize) {
    let start = CURRENT.load(Ordering::SeqCst);
    PEAK.store(start, Ordering::SeqCst);
    let result = run();
    (result, PEAK.load(Ordering::SeqCst).saturating_sub(start))
}

/// Bytes a parsed body keeps.
fn parsed_size(text: &str) -> (Value, usize) {
    let start = CURRENT.load(Ordering::SeqCst);
    let value: Value = serde_json::from_str(text).unwrap();
    let size = CURRENT.load(Ordering::SeqCst).saturating_sub(start);
    (value, size)
}

fn case_h() -> String {
    std::fs::read_to_string(format!(
        "{}/../../training-data/nta8800-public-comparison-h.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// Running an operation holds at most a fraction of an extra body: the
/// input member is moved out of the request, never copied. Before, the
/// service copied the body twice per request.
#[test]
fn an_operation_does_not_copy_the_request_body() {
    let _serial = SERIAL.lock().unwrap();
    // An input the typed route refuses at once: what is measured is the
    // service's own handling (edition, member checks, moving the member),
    // not kernel work, which differs per route.
    let text = json!({ "input": vec![0; 200_000] }).to_string();
    let (body, size) = parsed_size(&text);
    let op = operation("calculate_monthly_demand").unwrap();
    let (outcome, extra) = peak_during(|| execute(op, body));
    assert_eq!(outcome.status, 400, "{}", outcome.body);
    assert_eq!(outcome.body["code"], "invalid_request_shape");
    assert!(
        extra < size / 2,
        "running used {extra} B on top of a {size} B body: the body was copied"
    );
}

/// What a parsed body costs per JSON value, for the value cap: a compact
/// array element and a realistic project (public case H). Both stay under
/// 160 B per value, so the default cap of 500 000 values bounds a parsed
/// body at about 80 MB, whatever its byte size.
#[test]
fn a_parsed_value_costs_at_most_160_bytes() {
    let _serial = SERIAL.lock().unwrap();
    let array = json!(vec![0; 100_000]).to_string();
    let values = json_value_count(array.as_bytes(), usize::MAX).unwrap();
    let (_, size) = parsed_size(&array);
    let per_array_value = size as f64 / values as f64;

    let project = case_h();
    let values = json_value_count(project.as_bytes(), usize::MAX).unwrap();
    let (_, size) = parsed_size(&project);
    let per_project_value = size as f64 / values as f64;
    let per_project_byte = size as f64 / project.len() as f64;

    println!(
        "array: {per_array_value:.1} B per value; case H: {per_project_value:.1} B per value, {per_project_byte:.2} B per byte"
    );
    assert!(per_array_value <= 160.0, "{per_array_value}");
    assert!(per_project_value <= 160.0, "{per_project_value}");
    // The cap admits a project many times larger than the kernel benchmark.
    assert!(DEFAULT_MAX_JSON_VALUES >= 400 * values / 2);
}

/// The value count is an upper estimate that ignores brackets and commas
/// inside strings and stops at the limit.
#[test]
fn value_count_skips_strings_and_stops_at_the_limit() {
    assert_eq!(
        json_value_count(br#"{"a":"[,{,]","b":[1,2]}"#, 100),
        Some(5)
    );
    assert_eq!(json_value_count(br#""\",[""#, 100), Some(1));
    assert_eq!(json_value_count(b"[0,0,0,0]", 3), None);
}
