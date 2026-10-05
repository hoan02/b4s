//! Bounded FIFO admission for a device command transaction. A transaction can
//! include write and readback; transport success alone is not confirmation.

use std::{future::Future, time::Duration};
use tokio::sync::{Mutex, Semaphore};

pub struct CommandExecutor {
    admission: Semaphore,
    serial: Mutex<()>,
    deadline: Duration,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExecutionError {
    #[error("Device command queue is full; wait for the current operation")]
    QueueFull,
    #[error("Device command deadline expired; the device result is unknown")]
    Deadline,
    #[error("{0}")]
    Operation(String),
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new(16, Duration::from_secs(5))
    }
}

impl CommandExecutor {
    fn new(capacity: usize, deadline: Duration) -> Self {
        Self {
            admission: Semaphore::new(capacity),
            serial: Mutex::new(()),
            deadline,
        }
    }

    pub async fn run<T>(
        &self,
        work: impl Future<Output = Result<T, String>>,
    ) -> Result<T, ExecutionError> {
        let _admission = self
            .admission
            .try_acquire()
            .map_err(|_| ExecutionError::QueueFull)?;
        tokio::time::timeout(self.deadline, async {
            let _serial = self.serial.lock().await;
            work.await.map_err(ExecutionError::Operation)
        })
        .await
        .map_err(|_| ExecutionError::Deadline)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn queue_full_never_executes_rejected_work() {
        let executor = CommandExecutor::new(1, Duration::from_secs(1));
        let _occupied = executor.admission.acquire().await.unwrap();
        let result = executor
            .run(async {
                panic!("rejected command wrote a packet");
                #[allow(unreachable_code)]
                Ok(())
            })
            .await;
        assert_eq!(result, Err(ExecutionError::QueueFull));
    }

    #[tokio::test]
    async fn queued_deadline_never_starts_transport_and_releases_admission() {
        let executor = CommandExecutor::new(1, Duration::from_millis(10));
        let serial = executor.serial.lock().await;
        let result = executor
            .run(async {
                panic!("expired command wrote a packet");
                #[allow(unreachable_code)]
                Ok(())
            })
            .await;
        assert_eq!(result, Err(ExecutionError::Deadline));
        drop(serial);
        assert_eq!(executor.run(async { Ok(42) }).await.unwrap(), 42);
    }

    #[tokio::test]
    async fn failed_transport_is_not_success_and_does_not_poison_queue() {
        let executor = CommandExecutor::default();
        assert_eq!(
            executor
                .run(async { Err::<(), _>("disconnected".into()) })
                .await,
            Err(ExecutionError::Operation("disconnected".into()))
        );
        assert!(executor.run(async { Ok(()) }).await.is_ok());
    }
}
