use rppal::gpio::Gpio;
use rppal::gpio::Error;
use std::thread;
use std::time::Duration;

const CLOSE_LIMIT_PIN: u8 = 23;
const OPEN_LIMIT_PIN: u8 = 24;

const COUPLER_PIN: u8 = 25;

#[derive(PartialEq)]
enum DoorState {
    Closed,
    MovingUp,
    MovingDown,
    Open,
    Unknown
}

fn main() -> Result<(), Error> {
    let gpio = Gpio::new()?;

    let close_limit_switch = gpio.get(CLOSE_LIMIT_PIN)?.into_input();
    let open_limit_switch = gpio.get(OPEN_LIMIT_PIN)?.into_input();

    let mut coupler = gpio.get(COUPLER_PIN)?.into_output_low();

    let mut last_state = DoorState::Unknown;
    let mut state: DoorState;

    loop {
        let close_triggered = close_limit_switch.is_low();
        let open_triggered = open_limit_switch.is_low();

        state = if close_triggered && open_triggered {
            DoorState::Unknown
        } else if close_triggered {
            DoorState::Closed
        } else if open_triggered {
            DoorState::Open
        } else if last_state == DoorState::Closed {
            DoorState::MovingUp
        } else if last_state == DoorState::Open {
            DoorState::MovingDown
        } else {
            last_state
        };

        last_state = state;
        thread::sleep(Duration::from_millis(50));
    }
}
