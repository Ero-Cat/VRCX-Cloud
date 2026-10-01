use std::sync::{Arc, Mutex};

use crate::{OverlayActivityDelivery, OverlayActivitySink, OverlayActivitySnapshot};

#[derive(Clone, Default)]
pub struct OverlayActivitySinkRegistry {
    sinks: Arc<Mutex<Vec<Arc<dyn OverlayActivitySink>>>>,
}

impl OverlayActivitySinkRegistry {
    pub fn add(&self, sink: Arc<dyn OverlayActivitySink>) {
        match self.sinks.lock() {
            Ok(mut sinks) => sinks.push(sink),
            Err(error) => tracing::warn!("failed to lock overlay activity sinks: {error}"),
        }
    }

    fn sinks(&self) -> Vec<Arc<dyn OverlayActivitySink>> {
        self.sinks
            .lock()
            .map(|sinks| sinks.clone())
            .unwrap_or_else(|error| {
                tracing::warn!("failed to lock overlay activity sinks: {error}");
                Vec::new()
            })
    }
}

impl OverlayActivitySink for OverlayActivitySinkRegistry {
    fn emit_overlay_activity_snapshot(&self, snapshot: OverlayActivitySnapshot) {
        for sink in self.sinks() {
            sink.emit_overlay_activity_snapshot(snapshot.clone());
        }
    }

    fn emit_overlay_activity_delivery(&self, delivery: OverlayActivityDelivery) {
        for sink in self.sinks() {
            sink.emit_overlay_activity_delivery(delivery.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct CountingSink {
        snapshots: AtomicUsize,
        deliveries: AtomicUsize,
    }

    impl OverlayActivitySink for CountingSink {
        fn emit_overlay_activity_snapshot(&self, _snapshot: OverlayActivitySnapshot) {
            self.snapshots.fetch_add(1, Ordering::Relaxed);
        }

        fn emit_overlay_activity_delivery(&self, _delivery: OverlayActivityDelivery) {
            self.deliveries.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn fanout_delivers_each_event_to_every_registered_sink() {
        let registry = OverlayActivitySinkRegistry::default();
        let first = Arc::new(CountingSink {
            snapshots: AtomicUsize::new(0),
            deliveries: AtomicUsize::new(0),
        });
        let second = Arc::new(CountingSink {
            snapshots: AtomicUsize::new(0),
            deliveries: AtomicUsize::new(0),
        });
        registry.add(first.clone());
        registry.add(second.clone());

        registry.emit_overlay_activity_snapshot(OverlayActivitySnapshot::default());

        assert_eq!(first.snapshots.load(Ordering::Relaxed), 1);
        assert_eq!(second.snapshots.load(Ordering::Relaxed), 1);
        assert_eq!(first.deliveries.load(Ordering::Relaxed), 0);
    }
}
