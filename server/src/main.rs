use axum::{
    extract::{FromRequestParts, State}, http::{request::Parts, StatusCode}, response::{sse::Event, IntoResponse, Response, Sse}, routing::{get, post}, Json, RequestPartsExt, Router
};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use std::sync::{Arc, Mutex};
use futures::stream::Stream;
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::watch;
use serde::{Serialize, Deserialize};
use config::Config;
use embedded_hal::digital::{InputPin, OutputPin};
mod gpio;


#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    garage_door: GarageDoorConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct GarageDoorConfig {
    close_limit_pin: u8,
    open_limit_pin: u8,
    coupler_pin: u8,
    poll_interval_ms: u64,
    expected_shut_time_sec: u64,
    shut_time_buffer_sec: u64,
    coupler_duration_ms: u64,
    server_address: String,
    api_key: String,
}

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

#[derive(Debug, Clone, Copy)]
struct CommandStatus {
    executed: bool,
    pending: bool,
}

// Application state for Axum
#[derive(Debug, Clone)]
struct AppState {
    door_state: watch::Sender<DoorState>,
    latest_command: Arc<Mutex<Option<GpioCommand>>>,
    command_status: Arc<Mutex<CommandStatus>>,
}

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

struct Authenticated;

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing authorization header").into_response())?;

        let config = parts
            .extensions
            .get::<AppConfig>()
            .expect("AppConfig missing in extensions");

        if bearer.token() != config.garage_door.api_key {
            return Err((StatusCode::UNAUTHORIZED, "Invalid API key").into_response());
        }

        Ok(Authenticated)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize::<AppConfig>()?;

    // Convert config durations
    let poll_interval = Duration::from_millis(config.garage_door.poll_interval_ms);
    let expected_shut_time = Duration::from_secs(config.garage_door.expected_shut_time_sec);
    let shut_time_buffer = Duration::from_secs(config.garage_door.shut_time_buffer_sec);
    let coupler_duration = Duration::from_millis(config.garage_door.coupler_duration_ms);

    // Initialize GPIO components with config values
    let (close_limit_switch, open_limit_switch, coupler) = gpio::create_pins(
        config.garage_door.close_limit_pin,
        config.garage_door.open_limit_pin,
        config.garage_door.coupler_pin,
        poll_interval,
        expected_shut_time
    )?;

    // Create communication channels
    let (door_state_tx, _) = watch::channel(DoorState::Unknown);
    let app_state = AppState {
        door_state: door_state_tx.clone(),
        latest_command: Arc::new(Mutex::new(None)),
        command_status: Arc::new(Mutex::new(CommandStatus { executed: false, pending: false })),
    };

    monitor_gpio(
        close_limit_switch,
        open_limit_switch,
        coupler,
        door_state_tx,
        app_state.latest_command.clone(),
        app_state.command_status.clone(),
        poll_interval,
        expected_shut_time + shut_time_buffer,
        coupler_duration
    );

    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/toggle", post(toggle_door))
        .route("/open", post(open_door))
        .route("/close", post(close_door))
        .with_state(app_state)
        .layer(axum::Extension(config.clone()));

    let listener = tokio::net::TcpListener::bind(&config.garage_door.server_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// GPIO monitoring and control thread
// Modify monitor_gpio to use generic types
fn monitor_gpio<I1, I2, O>(
    mut close_limit: I1,
    mut open_limit: I2,
    mut coupler: O,
    state_tx: watch::Sender<DoorState>,
    latest_command: Arc<Mutex<Option<GpioCommand>>>,
    command_status: Arc<Mutex<CommandStatus>>,
    poll_interval: Duration,
    expected_shut_time: Duration,
    coupler_duration: Duration,
) where
    I1: InputPin + Send + 'static,
    I2: InputPin + Send + 'static,
    O: OutputPin + Send + 'static,
{
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

                    // Update command status
                    let mut status = command_status.lock().unwrap();
                    if should_activate {
                        let _ = coupler.set_high();
                        coupler_active = true;
                        coupler_start = Instant::now();
                        status.executed = true;
                    } else {
                        status.executed = false;
                    }
                    status.pending = false;
                }
            }

            // Update door state with timing consideration
            let now = Instant::now();
            let close_triggered = close_limit.is_low().unwrap_or(false);
            let open_triggered = open_limit.is_low().unwrap_or(false);

            let new_state = calculate_state(
                close_triggered,
                open_triggered,
                last_state,
                &mut last_known,
                now,
                expected_shut_time
            );

            if new_state != last_state {
                state_tx.send_replace(new_state);
                last_state = new_state;
            }

            // Manage coupler timing
            if coupler_active && now.duration_since(coupler_start) >= coupler_duration {
                let _ = coupler.set_low();
                coupler_active = false;
            }

            thread::sleep(poll_interval);
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
    expected_shut_time: Duration
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
            if current_time.duration_since(*last_known) > expected_shut_time {
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
    _: Authenticated,
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
    _: Authenticated,
    State(app_state): State<AppState>,
) -> (StatusCode, Json<DoorResponse>) {
    store_command(app_state.latest_command, app_state.command_status.clone(), GpioCommand::Toggle).await
}

async fn open_door(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> (StatusCode, Json<DoorResponse>) {
    store_command(app_state.latest_command, app_state.command_status.clone(), GpioCommand::Open).await
}

async fn close_door(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> (StatusCode, Json<DoorResponse>) {
    store_command(app_state.latest_command, app_state.command_status.clone(), GpioCommand::Close).await
}

// Update store_command to be async and wait for execution status
async fn store_command(
    cmd_mutex: Arc<Mutex<Option<GpioCommand>>>,
    status_mutex: Arc<Mutex<CommandStatus>>,
    cmd: GpioCommand,
) -> (StatusCode, Json<DoorResponse>) {
    // Set pending status and store command
    {
        let mut status = status_mutex.lock().unwrap();
        status.pending = true;
        status.executed = false;
    }
    
    {
        let mut lock = cmd_mutex.lock().unwrap();
        *lock = Some(cmd);
    }
    
    // Wait for command to be processed (with timeout)
    let start = Instant::now();
    let timeout = Duration::from_secs(2); // 2 second timeout
    
    while start.elapsed() < timeout {
        {
            let status = status_mutex.lock().unwrap();
            if !status.pending {
                // Command has been processed
                return if status.executed {
                    (
                        StatusCode::OK,
                        Json(DoorResponse {
                            status: "success",
                            message: "Command executed",
                        }),
                    )
                } else {
                    (
                        StatusCode::OK,
                        Json(DoorResponse {
                            status: "not_executed",
                            message: "Command not applicable in current state",
                        }),
                    )
                };
            }
        }
        
        // Small delay to prevent tight loop
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Timeout occurred
    (
        StatusCode::ACCEPTED,
        Json(DoorResponse {
            status: "pending",
            message: "Command queued but execution status unknown",
        }),
    )
}
