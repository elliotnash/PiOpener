use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, post},
    Router, Json,
};
use std::sync::{Arc, Mutex};
use futures::stream::Stream;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::watch;
use serde::Serialize;
use axum::http::StatusCode;

// State tracking and GPIO command enums
#[derive(Debug, Clone, Copy, PartialEq)]
enum DoorState {
    Unknown,
    Closed,
    Open,
    Ajar,
    MovingUp,
    MovingDown,
}

#[derive(Debug)]
enum GpioCommand {
    Toggle,
    Open,
    Close,
}

// Application state for Axum
#[derive(Debug, Clone)]
struct AppState {
    door_state: watch::Sender<DoorState>,
    latest_command: Arc<Mutex<Option<GpioCommand>>>,
}

// Constants
const CLOSE_LIMIT_PIN: u8 = 23;
const OPEN_LIMIT_PIN: u8 = 24;
const COUPLER_PIN: u8 = 25;
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const EXPECTED_SHUT_TIME: Duration = Duration::from_secs(18);
const COUPLER_DURATION: Duration = Duration::from_millis(100);

impl DoorState {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    
    // Initialize GPIO components
    let close_limit_switch = gpio.get(CLOSE_LIMIT_PIN)?.into_input();
    let open_limit_switch = gpio.get(OPEN_LIMIT_PIN)?.into_input();
    let coupler = gpio.get(COUPLER_PIN)?.into_output_low();

    // Create communication channels
    let (door_state_tx, _) = watch::channel(DoorState::Unknown);
    let latest_command = Arc::new(Mutex::new(None));

    monitor_gpio(
        close_limit_switch,
        open_limit_switch,
        coupler,
        door_state_tx.clone(),
        Arc::clone(&latest_command),
    );

    let app = Router::new()
        .route("/sse", get(status_handler))
        .route("/toggle", post(toggle_door))
        .route("/open", post(open_door))
        .route("/close", post(close_door))
        .with_state(AppState {
            door_state: door_state_tx,
            latest_command,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// GPIO monitoring and control thread
fn monitor_gpio(
    close_limit: InputPin,
    open_limit: InputPin,
    mut coupler: OutputPin,
    state_tx: watch::Sender<DoorState>,
    latest_command: Arc<Mutex<Option<GpioCommand>>>,
) {
    thread::spawn(move || {
        let mut last_state = DoorState::Unknown;
        let mut last_known = Instant::now();
        let mut coupler_active = false;
        let mut coupler_start = Instant::now();

        loop {
            // Process commands first
            let command = latest_command.lock().unwrap().take();
            if let Some(cmd) = command {
                if !coupler_active {
                    let should_activate = match cmd {
                        GpioCommand::Toggle => true,
                        GpioCommand::Open => last_state == DoorState::Closed,
                        GpioCommand::Close => last_state == DoorState::Open,
                    };

                    if should_activate {
                        coupler.set_high();
                        coupler_active = true;
                        coupler_start = Instant::now();
                    }
                }
            }

            // Update door state with timing consideration
            let now = Instant::now();
            let close_triggered = close_limit.is_low();
            let open_triggered = open_limit.is_low();

            let new_state = calculate_state(
                close_triggered,
                open_triggered,
                last_state,
                &mut last_known,
                now,
            );

            if new_state != last_state {
                state_tx.send(new_state).expect("Failed to send state");
                last_state = new_state;
            }

            // Manage coupler timing
            if coupler_active && now.duration_since(coupler_start) >= COUPLER_DURATION {
                coupler.set_low();
                coupler_active = false;
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}

// Door state calculation logic
fn calculate_state(
    close_triggered: bool,
    open_triggered: bool,
    last_state: DoorState,
    last_known: &mut Instant,
    current_time: Instant,
) -> DoorState {
    match (close_triggered, open_triggered) {
        (true, true) => DoorState::Unknown,
        (true, false) => {
            *last_known = current_time;
            DoorState::Closed
        }
        (false, true) => {
            *last_known = current_time;
            DoorState::Open
        }
        (false, false) => {
            if current_time.duration_since(*last_known) > EXPECTED_SHUT_TIME {
                DoorState::Ajar
            } else {
                match last_state {
                    DoorState::Closed => DoorState::MovingUp,
                    DoorState::Open => DoorState::MovingDown,
                    _ => last_state,
                }
            }
        }
    }
}

// Axum handlers
async fn status_handler(
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let mut rx = app_state.door_state.subscribe();
    let stream = async_stream::try_stream! {
        let initial = *rx.borrow();
        yield Event::default().data(initial.value());

        while let Ok(()) = rx.changed().await {
            let current = *rx.borrow();
            yield Event::default().data(current.value());
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[derive(Serialize)]
struct DoorResponse {
    status: &'static str,
    message: &'static str,
}

async fn toggle_door(
    State(app_state): State<AppState>,
) -> (StatusCode, Json<DoorResponse>) {
    store_command(app_state.latest_command, GpioCommand::Toggle)
}

async fn open_door(
    State(app_state): State<AppState>,
) -> (StatusCode, Json<DoorResponse>) {
    store_command(app_state.latest_command, GpioCommand::Open)
}

async fn close_door(
    State(app_state): State<AppState>,
) -> (StatusCode, Json<DoorResponse>) {
    store_command(app_state.latest_command, GpioCommand::Close)
}

fn store_command(
    cmd_mutex: Arc<Mutex<Option<GpioCommand>>>,
    cmd: GpioCommand,
) -> (StatusCode, Json<DoorResponse>) {
    let mut lock = cmd_mutex.lock().unwrap();
    *lock = Some(cmd);
    
    (
        StatusCode::OK,
        Json(DoorResponse {
            status: "success",
            message: "Command queued",
        }),
    )
}
