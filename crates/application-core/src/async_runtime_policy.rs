#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TokioThreadCounts {
    worker_threads: usize,
    max_blocking_threads: usize,
}

const MIN_WORKER_THREADS: usize = 2;
const MAX_WORKER_THREADS: usize = 8;
const MAX_BLOCKING_THREADS: usize = 64;

pub fn recommended_tokio_worker_threads_for(logical_cpus: usize) -> usize {
    recommended_tokio_thread_counts_for(logical_cpus).worker_threads
}

pub fn recommended_tokio_worker_threads() -> usize {
    recommended_tokio_worker_threads_for(available_logical_cpus())
}

pub fn recommended_tokio_max_blocking_threads_for(logical_cpus: usize) -> usize {
    recommended_tokio_thread_counts_for(logical_cpus).max_blocking_threads
}

pub fn recommended_tokio_max_blocking_threads() -> usize {
    recommended_tokio_max_blocking_threads_for(available_logical_cpus())
}

fn recommended_tokio_thread_counts_for(logical_cpus: usize) -> TokioThreadCounts {
    TokioThreadCounts {
        worker_threads: logical_cpus.clamp(MIN_WORKER_THREADS, MAX_WORKER_THREADS),
        max_blocking_threads: MAX_BLOCKING_THREADS,
    }
}

fn available_logical_cpus() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_threads_track_logical_cpus_between_the_floor_and_the_cap() {
        let cases = [
            (0, MIN_WORKER_THREADS),
            (1, MIN_WORKER_THREADS),
            (2, MIN_WORKER_THREADS),
            (3, 3),
            (4, 4),
            (5, 5),
            (8, MAX_WORKER_THREADS),
            (16, MAX_WORKER_THREADS),
            (64, MAX_WORKER_THREADS),
        ];

        for (logical_cpus, expected_worker_threads) in cases {
            assert_eq!(
                recommended_tokio_worker_threads_for(logical_cpus),
                expected_worker_threads
            );
        }
    }

    #[test]
    fn blocking_thread_cap_does_not_depend_on_logical_cpus() {
        for logical_cpus in [0, 1, 2, 4, 8, 64] {
            assert_eq!(
                recommended_tokio_max_blocking_threads_for(logical_cpus),
                MAX_BLOCKING_THREADS
            );
        }
    }
}
