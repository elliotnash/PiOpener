use rppal::gpio::Gpio;
use rppal::gpio::Error;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const CLOSE_LIMIT_PIN: u8 = 23;
const OPEN_LIMIT_PIN: u8 = 24;

const COUPLER_PIN: u8 = 25;

const EXPECTED_SHUT_TIME: Duration = Duration::from_secs(15);

const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(PartialEq, Debug, Clone, Copy)]
enum DoorState {
    Closed,
    MovingUp,
    MovingDown,
    Open,
    Ajar,
    Unknown
}

fn main() -> Result<(), Error> {
    let gpio = Gpio::new()?;

    let close_limit_switch = gpio.get(CLOSE_LIMIT_PIN)?.into_input();
    let open_limit_switch = gpio.get(OPEN_LIMIT_PIN)?.into_input();

    let mut coupler = gpio.get(COUPLER_PIN)?.into_output_low();

    let mut last_state = DoorState::Unknown;
    let mut state: DoorState;

    let mut last_known = Instant::now();

    loop {
        let close_triggered = close_limit_switch.is_low();
        let open_triggered = open_limit_switch.is_low();

        let test = last_known - Instant::now();

        state = if close_triggered && open_triggered {
            DoorState::Unknown
        } else if close_triggered {
            last_known = Instant::now();
            DoorState::Closed
        } else if open_triggered {
            last_known = Instant::now();
            DoorState::Open
        } else if (last_known - Instant::now()) > EXPECTED_SHUT_TIME {
            DoorState::Ajar
        } else if last_state == DoorState::Closed {
            DoorState::MovingUp
        } else if last_state == DoorState::Open {
            DoorState::MovingDown
        } else {
            last_state
        };

        if state != last_state {
            println!("State changed - state: {:?}, Close limit: {}, open limit: {}", state, close_triggered, open_triggered);
        }

        last_state = state;
        thread::sleep(POLL_INTERVAL);
    }
}
