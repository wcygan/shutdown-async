#[cfg(test)]
mod tests {
    use shutdown_async::ShutdownController;

    #[tokio::test]
    async fn shutdown_completes() {
        let shutdown = ShutdownController::new();

        let t = tokio::spawn({
            let mut monitor = shutdown.subscribe();
            async move {
                monitor.recv().await;
            }
        });

        shutdown.shutdown().await;
        assert!(t.await.is_ok());
    }

    #[tokio::test]
    async fn monitor_is_not_ready_for_shutdown() {
        let shutdown = ShutdownController::new();
        let monitor = shutdown.subscribe();
        assert!(!monitor.is_shutdown());
    }

    #[tokio::test]
    async fn monitor_is_not_ready_for_shutdown2() {
        let shutdown = ShutdownController::new();

        let t = tokio::spawn({
            let mut monitor = shutdown.subscribe();
            async move {
                assert!(!monitor.is_shutdown());
                monitor.recv().await;
            }
        });

        shutdown.shutdown().await;
        assert!(t.await.is_ok());
    }

    #[tokio::test]
    async fn monitor_is_ready_for_shutdown() {
        let shutdown = ShutdownController::new();

        let t = tokio::spawn({
            let mut monitor = shutdown.subscribe();
            async move {
                monitor.recv().await;
                assert!(monitor.is_shutdown());
            }
        });

        shutdown.shutdown().await;
        assert!(t.await.is_ok());
    }
}
