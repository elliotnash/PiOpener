use axum::{
    extract::{FromRequestParts, State}, http::{request::Parts, StatusCode}, response::{sse::Event, IntoResponse, Response, Sse}, routing::{get, post}, Json, RequestPartsExt, Router
};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use std::{collections::VecDeque, sync::{Arc, Mutex}};
use futures::stream::Stream;
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::watch;
use serde::Serialize;
use embedded_hal::digital::{InputPin, OutputPin, PinState};
use config::AppConfig;

mod gpio;
mod config;

#[derive(Debug, Clone, Copy, PartialEq)]
struct DoorState {
    status: DoorStatus,
    setpoint: DoorSetpoint,
    position: f64,
}

// State tracking and GPIO command enums
#[derive(Debug, Clone, Copy, PartialEq)]
enum DoorStatus {
    Closed,
    Open,
    Ajar,
    MovingUp,
    MovingDown,
}

impl DoorStatus {
    fn value(&self) -> &'static str {
        match self {
            DoorStatus::Closed => "closed",
            DoorStatus::Open => "open",
            DoorStatus::Ajar => "ajar",
            DoorStatus::MovingUp => "moving_up",
            DoorStatus::MovingDown => "moving_down",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DoorSetpoint {
    Closed,
    Open,
    Ajar,
}

impl DoorSetpoint {
    fn value(&self) -> &'static str {
        match self {
            DoorSetpoint::Closed => "closed",
            DoorSetpoint::Open => "open",
            DoorSetpoint::Ajar => "ajar",
        }
    }
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
    let config = config::load_config()?;

    // Convert config durations
    let poll_interval = Duration::from_millis(config.garage_door.poll_interval_ms);
    let expected_shut_time = Duration::from_secs(config.garage_door.expected_shut_time_sec);
    let _shut_time_buffer = Duration::from_secs(config.garage_door.shut_time_buffer_sec);
    let limit_cooldown = Duration::from_millis(config.garage_door.limit_cooldown_ms);

    // Initialize GPIO components with config values
    let (close_limit_switch, open_limit_switch, coupler) = gpio::create_pins(
        config.garage_door.close_limit_pin,
        config.garage_door.open_limit_pin,
        config.garage_door.coupler_pin,
        poll_interval,
        expected_shut_time
    )?;

    // Create communication channels
    let (door_state_tx, _) = watch::channel(DoorState{
        status: DoorStatus::Ajar,
        setpoint: DoorSetpoint::Ajar,
        position: 0_f64,
    });
    let app_state = AppState {
        door_state: door_state_tx.clone(),
        latest_command: Arc::new(Mutex::new(None)),
    };

    monitor_gpio(
        close_limit_switch,
        open_limit_switch,
        coupler,
        door_state_tx,
        app_state.latest_command.clone(),
        poll_interval,
        expected_shut_time,
        config.garage_door.coupler_active_low,
        config.garage_door.coupler_active_intervals,
        config.garage_door.coupler_rest_intervals,
        limit_cooldown,
    );

    let app = Router::new()
        .route("/watch-status", get(watch_status_handler))
        .route("/status", get(current_status_handler))
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
    poll_interval: Duration,
    expected_shut_time: Duration,
    coupler_active_low: bool,
    coupler_active_intervals: u64,
    coupler_rest_intervals: u64,
    limit_cooldown: Duration,
) where
    I1: InputPin + Send + 'static,
    I2: InputPin + Send + 'static,
    O: OutputPin + Send + 'static,
{
    thread::spawn(move || {
        let close_triggered = close_limit.is_low().unwrap_or(false);
        let open_triggered = open_limit.is_low().unwrap_or(false);

        let mut last_state = DoorState {
            status: match (close_triggered, open_triggered) {
                (true, false) => DoorStatus::Closed,
                (false, true) => DoorStatus::Open,
                _ => DoorStatus::Ajar,
            },
            setpoint: match (close_triggered, open_triggered) {
                (true, false) => DoorSetpoint::Closed,
                (false, true) => DoorSetpoint::Open,
                _ => DoorSetpoint::Ajar,
            },
            position: 0.0,
        };
        state_tx.send_replace(last_state);

        let mut last_direction = 0_f64;
        let mut last_time = Instant::now();

        let mut coupler_queue: VecDeque<PinState> = VecDeque::with_capacity(10);

        let mut last_full_close = Instant::now();
        let mut last_full_open = Instant::now();

        let start_state = if coupler_active_low {
            PinState::Low
        } else {
            PinState::High
        };

        let end_state = if coupler_active_low {
            PinState::High
        } else {
            PinState::Low
        };

        let toggle_coupler = |queue: &mut VecDeque<PinState>, queue_intervals: u64| {
            for _ in 0..queue_intervals {
                queue.push_back(start_state);
            }
            for _ in 0..queue_intervals {
                queue.push_back(end_state);
            }
        };

        let rest_coupler = |queue: &mut VecDeque<PinState>, rest_intervals: u64| {
            for _ in 0..rest_intervals {
                queue.push_back(end_state);
            }
        };

        loop {
            // Update door state with timing consideration
            let now = Instant::now();
            let close_triggered = close_limit.is_low().unwrap_or(false);
            let open_triggered = open_limit.is_low().unwrap_or(false);

            let mut new_state = match (close_triggered, open_triggered) {
                (true, true) => {
                    DoorState {
                        status: DoorStatus::Ajar,
                        setpoint: last_state.setpoint,
                        position: last_state.position,
                    }
                },
                (true, false) => {
                    if now.duration_since(last_full_open) < limit_cooldown {
                        last_state
                    } else {
                        last_direction = -1_f64;
                        DoorState {
                            status: DoorStatus::Closed,
                            setpoint: last_state.setpoint,
                            position: 0_f64,
                        }
                    }  
                },
                (false, true) => {
                    if now.duration_since(last_full_close) < limit_cooldown {
                        last_state
                    } else {
                        last_direction = 1_f64;
                        DoorState {
                            status: DoorStatus::Open,
                            setpoint: last_state.setpoint,
                            position: 1_f64,
                        }
                    }
                },
                (false, false) => {
                    match last_state.status {
                        DoorStatus::Closed => DoorState {
                            status: DoorStatus::MovingUp,
                            setpoint: last_state.setpoint,
                            position: (last_state.position + last_direction * (now.duration_since(last_time).as_secs_f64() / expected_shut_time.as_secs_f64())).clamp(0_f64, 1_f64),
                        },
                        DoorStatus::Open => DoorState {
                            status: DoorStatus::MovingDown,
                            setpoint: last_state.setpoint,
                            position: (last_state.position + last_direction * (now.duration_since(last_time).as_secs_f64() / expected_shut_time.as_secs_f64())).clamp(0_f64, 1_f64),
                        },
                        DoorStatus::MovingUp | DoorStatus::MovingDown => DoorState {
                            status: last_state.status,
                            setpoint: last_state.setpoint,
                            position: (last_state.position + last_direction * (now.duration_since(last_time).as_secs_f64() / expected_shut_time.as_secs_f64())).clamp(0_f64, 1_f64),
                        },
                        _ => {
                            last_state
                        },
                    }
                }
            };

            // Process commands
            let command = latest_command.lock().unwrap().take();
            if let Some(cmd) = command {
                match cmd {
                    GpioCommand::Toggle => {
                        // If toggled we will always activate coupler exactly once
                        toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                        // We will invert setpoint. If stopped, we will use the direction opposite the last direction.
                        new_state = match new_state.status {
                            DoorStatus::Closed => {
                                last_full_open = now;
                                last_direction = 1_f64;
                                DoorState {
                                    status: DoorStatus::MovingUp,
                                    setpoint: DoorSetpoint::Open,
                                    position: new_state.position,
                                }
                            },
                            DoorStatus::Open => {
                                last_full_close = now;
                                last_direction = -1_f64;
                                DoorState {
                                    status: DoorStatus::MovingDown,
                                    setpoint: DoorSetpoint::Closed,
                                    position: new_state.position,
                                }
                            },
                            DoorStatus::MovingUp | DoorStatus::MovingDown => DoorState {
                                status: DoorStatus::Ajar,
                                setpoint: DoorSetpoint::Ajar,
                                position: new_state.position,
                            },
                            _ => if last_direction > 0_f64 {
                                last_direction = -1_f64;
                                DoorState {
                                    status: DoorStatus::MovingDown,
                                    setpoint: DoorSetpoint::Closed,
                                    position: new_state.position,
                                }
                            } else {
                                last_direction = 1_f64;
                                DoorState {
                                    status: DoorStatus::MovingUp,
                                    setpoint: DoorSetpoint::Open,
                                    position: new_state.position,
                                }
                            }
                        };
                    },
                    GpioCommand::Open => {
                        // If command is open or close then we need to decide if we need 0, 1, 2, or 3 clicks
                        new_state = match new_state.status {
                            DoorStatus::MovingDown => {
                                toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                rest_coupler(&mut coupler_queue, coupler_rest_intervals);
                                toggle_coupler(&mut coupler_queue, coupler_active_intervals);

                                DoorState {
                                    status: DoorStatus::MovingUp,
                                    setpoint: DoorSetpoint::Open,
                                    position: new_state.position,
                                }
                            },
                            DoorStatus::Ajar => {
                                // If it went up last time, now it will go down, so we need three clicks. Otherwise we just need 1
                                if last_direction > 0_f64 {
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                    rest_coupler(&mut coupler_queue, coupler_rest_intervals);
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                } else {
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                }
                                DoorState {
                                    status: DoorStatus::MovingUp,
                                    setpoint: DoorSetpoint::Open,
                                    position: new_state.position,
                                }
                            },
                            DoorStatus::Closed => {
                                last_full_open = now;
                                toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                DoorState {
                                    status: DoorStatus::MovingUp,
                                    setpoint: DoorSetpoint::Open,
                                    position: new_state.position,
                                }
                            }
                            _ => DoorState {
                                status: new_state.status,
                                setpoint: DoorSetpoint::Open,
                                position: new_state.position,
                            }
                        };
                        // In all cases, we are now moving up (hopefully)
                        last_direction = 1_f64;
                    },
                    GpioCommand::Close => {
                        // If command is close then we need to decide if we need 0, 1, 2, or 3 clicks
                        new_state = match new_state.status {
                            DoorStatus::MovingUp => {
                                toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                rest_coupler(&mut coupler_queue, coupler_rest_intervals);
                                toggle_coupler(&mut coupler_queue, coupler_active_intervals);

                                DoorState {
                                    status: DoorStatus::MovingDown,
                                    setpoint: DoorSetpoint::Closed,
                                    position: new_state.position,
                                }
                            },
                            DoorStatus::Ajar => {
                                // If it went down last time, now it will go up, so we need three clicks. Otherwise we just need 1
                                if last_direction < 0_f64 {
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                    rest_coupler(&mut coupler_queue, coupler_rest_intervals);
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                } else {
                                    toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                }
                                DoorState {
                                    status: DoorStatus::MovingDown,
                                    setpoint: DoorSetpoint::Closed,
                                    position: new_state.position,
                                }
                            },
                            DoorStatus::Open => {
                                last_full_close = now;
                                toggle_coupler(&mut coupler_queue, coupler_active_intervals);
                                DoorState {
                                    status: DoorStatus::MovingDown,
                                    setpoint: DoorSetpoint::Closed,
                                    position: new_state.position,
                                }
                            }
                            _ => DoorState {
                                status: new_state.status,
                                setpoint: DoorSetpoint::Closed,
                                position: new_state.position,
                            }
                        };
                        // In all cases we should be moving down
                        last_direction = -1_f64;
                    }
                }
            }

             // Toggle coupler if requested
             if let Some(pin_state) = coupler_queue.pop_front() {
                let _ = coupler.set_state(pin_state);
            }

            if new_state != last_state {
                state_tx.send_replace(new_state);
                last_state = new_state;
            }
            last_time = now;

            thread::sleep(poll_interval);
        }
    });
}

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
    setpoint: &'static str,
    position: f64,
}

// Axum handlers
async fn watch_status_handler(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let mut rx = app_state.door_state.subscribe();
    let stream = async_stream::try_stream! {
        let initial = *rx.borrow();
        yield Event::default().json_data(StatusResponse { status: initial.status.value(), setpoint: initial.setpoint.value(), position: initial.position }).unwrap();

        while let Ok(()) = rx.changed().await {
            let current = *rx.borrow();
            yield Event::default().json_data(StatusResponse { status: current.status.value(), setpoint: current.setpoint.value(), position: current.position }).unwrap();
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

// Handler to get current door status without streaming
async fn current_status_handler(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> Json<StatusResponse> {
    let rx = app_state.door_state.subscribe();
    let current = *rx.borrow();
    Json(StatusResponse { status: current.status.value(), setpoint: current.setpoint.value(), position: current.position })
}

#[derive(Serialize)]
struct DoorResponse {
    status: &'static str,
    message: &'static str,
}

async fn toggle_door(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> Json<DoorResponse> {
    store_command(app_state.latest_command, GpioCommand::Toggle).await
}

async fn open_door(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> Json<DoorResponse> {
    store_command(app_state.latest_command, GpioCommand::Open).await
}

async fn close_door(
    _: Authenticated,
    State(app_state): State<AppState>,
) -> Json<DoorResponse> {
    store_command(app_state.latest_command, GpioCommand::Close).await
}

// Update store_command to be async and wait for execution status
async fn store_command(
    cmd_mutex: Arc<Mutex<Option<GpioCommand>>>,
    cmd: GpioCommand,
) -> Json<DoorResponse> {
    // Set pending status and store command    
    {
        let mut lock = cmd_mutex.lock().unwrap();
        *lock = Some(cmd);
    }
    
    Json(DoorResponse {
        status: "success",
        message: "Command executed",
    })
}
