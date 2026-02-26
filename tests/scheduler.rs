use heol::scheduler::Scheduler;
use heol::solar::SolarEngine;
use tokio::sync::watch;
use std::time::Duration;

#[tokio::test]
async fn scheduler_broadcasts_solar_state() {
    let engine = SolarEngine::new(48.8566, 2.3522, None);
    let (tx, rx) = watch::channel(None);

    let scheduler = Scheduler::new(engine, tx, Duration::from_millis(50));

    let handle = tokio::spawn(async move {
        scheduler.run_once().await;
    });

    handle.await.unwrap();

    let state = rx.borrow().clone();
    assert!(state.is_some(), "scheduler should have broadcast a SolarState");
    let state = state.unwrap();
    // Should be a valid elevation for current time
    assert!(state.elevation > -90.0 && state.elevation < 90.0);
}
