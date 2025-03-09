use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use rppal::gpio::{Gpio, InputPin};
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::watch;


const CLOSE_LIMIT_PIN: u8 = 23;
const OPEN_LIMIT_PIN: u8 = 24;
const COUPLER_PIN: u8 = 25;

const EXPECTED_SHUT_TIME: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq)]
enum DoorState {
    Unknown,
    Closed,
    Open,
    Ajar,
    MovingUp,
    MovingDown,
}

impl DoorState {
    // Returns string representation of the state
    fn value(&self) -> &'static str {
        match self {
            DoorState::Unknown => "unknown",
            DoorState::Closed => "closed",
            DoorState::Open => "open",
            DoorState::Ajar => "ajar",
            DoorState::MovingUp => "moving_up",
            DoorState::MovingDown => "moving_down",
        }
    }
}

impl std::fmt::Display for DoorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    
    let close_limit_switch = gpio.get(CLOSE_LIMIT_PIN)?.into_input();
    let open_limit_switch = gpio.get(OPEN_LIMIT_PIN)?.into_input();
    let _coupler = gpio.get(COUPLER_PIN)?.into_output_low();

    let (state_tx, _) = watch::channel(DoorState::Unknown);

    // Spawn GPIO monitoring with configured parameters
    monitor_gpio(
        close_limit_switch,
        open_limit_switch,
        state_tx.clone(),
        POLL_INTERVAL,
        EXPECTED_SHUT_TIME,
    );

    let app = Router::new()
        .route("/status", get(status_handler))
        .with_state(state_tx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Separate GPIO monitoring function
fn monitor_gpio(
    close_limit_switch: InputPin,
    open_limit_switch: InputPin,
    state_tx: watch::Sender<DoorState>,
    poll_interval: Duration,
    expected_shut_time: Duration,
) {
    thread::spawn(move || {
        let mut last_state = DoorState::Unknown;
        let mut state;
        let mut last_known = Instant::now();

        loop {
            let close_triggered = close_limit_switch.is_low();
            let open_triggered = open_limit_switch.is_low();

            state = if close_triggered && open_triggered {
                DoorState::Unknown
            } else if close_triggered {
                last_known = Instant::now();
                DoorState::Closed
            } else if open_triggered {
                last_known = Instant::now();
                DoorState::Open
            } else if (Instant::now() - last_known) > expected_shut_time {
                DoorState::Ajar
            } else if last_state == DoorState::Closed {
                DoorState::MovingUp
            } else if last_state == DoorState::Open {
                DoorState::MovingDown
            } else {
                last_state
            };

            if state != last_state {
                state_tx.send(state).expect("Failed to send state update");
                last_state = state;
            }

            thread::sleep(poll_interval);
        }
    });
}

async fn status_handler(
    State(state_tx): State<watch::Sender<DoorState>>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let mut rx = state_tx.subscribe();

    let stream = async_stream::try_stream! {
        let initial = *rx.borrow();
        yield Event::default().data(initial.value());

        while let Ok(()) = rx.changed().await {
            let current = *rx.borrow();
            yield Event::default().data(current.value());
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1)))
}
