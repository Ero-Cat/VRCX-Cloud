use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use serde_json::json;

use crate::RuntimeEventBus;
use crate::{Error, Result};

use super::{OverflowPolicy, RuntimeWorker, RuntimeWorkerOptions};

#[test]
fn processes_batches_in_order() -> Result<()> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let worker_seen = Arc::clone(&seen);
    let worker = RuntimeWorker::start(
        "test-order",
        RuntimeWorkerOptions {
            max_batch: 2,
            flush_interval: Duration::from_millis(1),
            ..Default::default()
        },
        RuntimeEventBus::new(),
        move |batch: Vec<i32>| {
            worker_seen.lock().unwrap().extend(batch);
            Ok(())
        },
    );

    worker.push_batch([1, 2, 3, 4, 5])?;
    assert!(worker.wait_until_idle(Duration::from_secs(2)));
    assert_eq!(*seen.lock().unwrap(), vec![1, 2, 3, 4, 5]);
    worker.stop();
    Ok(())
}

#[test]
fn drop_oldest_keeps_newest_items() -> Result<()> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let worker_seen = Arc::clone(&seen);
    let worker = RuntimeWorker::start(
        "test-drop-oldest",
        RuntimeWorkerOptions {
            capacity: 3,
            max_batch: 10,
            flush_interval: Duration::from_millis(100),
            overflow_policy: OverflowPolicy::DropOldest,
        },
        RuntimeEventBus::new(),
        move |batch: Vec<i32>| {
            worker_seen.lock().unwrap().extend(batch);
            Ok(())
        },
    );

    let report = worker.push_batch([1, 2, 3, 4, 5])?;
    assert_eq!(report.accepted, 5);
    assert_eq!(report.dropped, 2);
    assert!(worker.wait_until_idle(Duration::from_secs(2)));
    assert_eq!(*seen.lock().unwrap(), vec![3, 4, 5]);
    worker.stop();
    Ok(())
}

#[test]
fn continues_after_handler_error() -> Result<()> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let worker_seen = Arc::clone(&seen);
    let event_bus = RuntimeEventBus::new();
    let worker = RuntimeWorker::start(
        "test-error",
        RuntimeWorkerOptions {
            max_batch: 1,
            flush_interval: Duration::from_millis(1),
            ..Default::default()
        },
        event_bus.clone(),
        move |batch: Vec<i32>| {
            let value = batch[0];
            worker_seen.lock().unwrap().push(value);
            if value == 1 {
                Err(Error::Custom("expected test error".into()))
            } else {
                Ok(())
            }
        },
    );

    worker.push_batch([1, 2])?;
    assert!(worker.wait_until_idle(Duration::from_secs(2)));
    assert_eq!(*seen.lock().unwrap(), vec![1, 2]);
    let events = event_bus.take_events_for_test();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "runtimeWorkerError");
    assert_eq!(
        events[0].payload,
        json!({
            "worker": "test-error",
            "message": "expected test error"
        })
        .into()
    );
    worker.stop();
    Ok(())
}

#[test]
fn wait_until_idle_includes_the_active_handler() -> Result<()> {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let release_rx = Mutex::new(release_rx);
    let worker = RuntimeWorker::start(
        "test-active-handler",
        RuntimeWorkerOptions {
            max_batch: 1,
            flush_interval: Duration::ZERO,
            ..Default::default()
        },
        RuntimeEventBus::new(),
        move |_batch: Vec<i32>| {
            started_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
            Ok(())
        },
    );

    worker.push_batch([1])?;
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(!worker.wait_until_idle(Duration::from_millis(20)));

    release_tx.send(()).unwrap();
    assert!(worker.wait_until_idle(Duration::from_secs(2)));
    worker.stop();
    Ok(())
}

#[test]
fn backpressure_waits_for_capacity_without_dropping_work() -> Result<()> {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let release_rx = Mutex::new(release_rx);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let worker_seen = Arc::clone(&seen);
    let worker = Arc::new(RuntimeWorker::start(
        "test-backpressure",
        RuntimeWorkerOptions {
            capacity: 1,
            max_batch: 1,
            flush_interval: Duration::ZERO,
            overflow_policy: OverflowPolicy::Backpressure,
        },
        RuntimeEventBus::new(),
        move |batch: Vec<i32>| {
            let value = batch[0];
            if value == 1 {
                started_tx.send(()).unwrap();
                release_rx.lock().unwrap().recv().unwrap();
            }
            worker_seen.lock().unwrap().push(value);
            Ok(())
        },
    ));

    worker.push_batch([1])?;
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.push_batch([2])?;

    let producer_worker = Arc::clone(&worker);
    let (pushed_tx, pushed_rx) = mpsc::sync_channel(1);
    let producer = std::thread::spawn(move || {
        pushed_tx.send(producer_worker.push_batch([3])).unwrap();
    });
    assert!(pushed_rx.recv_timeout(Duration::from_millis(20)).is_err());

    release_tx.send(()).unwrap();
    let report = pushed_rx.recv_timeout(Duration::from_secs(2)).unwrap()?;
    assert_eq!(report.accepted, 1);
    assert_eq!(report.dropped, 0);
    producer.join().unwrap();
    assert!(worker.wait_until_idle(Duration::from_secs(2)));
    assert_eq!(*seen.lock().unwrap(), vec![1, 2, 3]);
    worker.stop();
    Ok(())
}

#[test]
fn stop_drains_already_accepted_work_before_returning() -> Result<()> {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let release_rx = Mutex::new(release_rx);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let worker_seen = Arc::clone(&seen);
    let worker = RuntimeWorker::start(
        "test-stop-drain",
        RuntimeWorkerOptions {
            max_batch: 1,
            flush_interval: Duration::ZERO,
            ..Default::default()
        },
        RuntimeEventBus::new(),
        move |batch: Vec<i32>| {
            let value = batch[0];
            if value == 1 {
                started_tx.send(()).unwrap();
                release_rx.lock().unwrap().recv().unwrap();
            }
            worker_seen.lock().unwrap().push(value);
            Ok(())
        },
    );

    worker.push_batch([1, 2])?;
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let (stopped_tx, stopped_rx) = mpsc::sync_channel(1);
    let stopper = std::thread::spawn(move || {
        worker.stop();
        stopped_tx.send(()).unwrap();
    });
    assert!(stopped_rx.recv_timeout(Duration::from_millis(20)).is_err());

    release_tx.send(()).unwrap();
    stopped_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    stopper.join().unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![1, 2]);
    Ok(())
}

#[test]
fn stop_wakes_a_backpressured_producer_with_a_stopped_error() -> Result<()> {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let release_rx = Mutex::new(release_rx);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let worker_seen = Arc::clone(&seen);
    let worker = Arc::new(RuntimeWorker::start(
        "test-stop-backpressure",
        RuntimeWorkerOptions {
            capacity: 1,
            max_batch: 1,
            flush_interval: Duration::ZERO,
            overflow_policy: OverflowPolicy::Backpressure,
        },
        RuntimeEventBus::new(),
        move |batch: Vec<i32>| {
            let value = batch[0];
            if value == 1 {
                started_tx.send(()).unwrap();
                release_rx.lock().unwrap().recv().unwrap();
            }
            worker_seen.lock().unwrap().push(value);
            Ok(())
        },
    ));

    worker.push_batch([1])?;
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.push_batch([2])?;

    let producer_worker = Arc::clone(&worker);
    let (pushed_tx, pushed_rx) = mpsc::sync_channel(1);
    let producer = std::thread::spawn(move || {
        pushed_tx.send(producer_worker.push_batch([3])).unwrap();
    });
    assert!(pushed_rx.recv_timeout(Duration::from_millis(20)).is_err());

    let stopper_worker = Arc::clone(&worker);
    let (stopped_tx, stopped_rx) = mpsc::sync_channel(1);
    let stopper = std::thread::spawn(move || {
        stopper_worker.stop();
        stopped_tx.send(()).unwrap();
    });
    let error = pushed_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("test-stop-backpressure worker is stopped"));

    release_tx.send(()).unwrap();
    stopped_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    producer.join().unwrap();
    stopper.join().unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![1, 2]);
    Ok(())
}
