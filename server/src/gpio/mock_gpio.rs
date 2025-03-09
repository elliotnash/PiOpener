use embedded_hal::digital::{InputPin, OutputPin, ErrorType, Error};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct MockInputPin {
    state: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct MockOutputPin {
    state: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct PinError;

impl Error for PinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl MockInputPin {
    pub fn new(initial_state: bool) -> Self {
        Self {
            state: Arc::new(AtomicBool::new(initial_state)),
        }
    }

    // Helper for tests to change the pin state
    pub fn set_state(&self, state: bool) {
        self.state.store(state, Ordering::SeqCst);
    }
}

impl ErrorType for MockInputPin {
    type Error = PinError;
}

impl InputPin for MockInputPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state.load(Ordering::SeqCst))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.state.load(Ordering::SeqCst))
    }
}

impl MockOutputPin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicBool::new(false)),
        }
    }

    // Helper for tests to read the pin state
    pub fn is_set_high(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }
}

impl ErrorType for MockOutputPin {
    type Error = PinError;
}

impl OutputPin for MockOutputPin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state.store(false, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
pub struct DoorSimulation {
    position: Arc<Mutex<f64>>,
    velocity: Arc<Mutex<f64>>,
    last_coupler_state: Arc<AtomicBool>,
    last_direction: Arc<Mutex<f64>>,
    last_update_time: Arc<Mutex<std::time::Instant>>,
    door_speed: f64, // Store the calculated door speed
}

impl DoorSimulation {
    fn new(expected_shut_time: Duration) -> Self {
        // Calculate speed as distance/time (1.0 units / expected_shut_time)
        let door_speed = 1.0 / expected_shut_time.as_secs_f64();
        
        Self {
            position: Arc::new(Mutex::new(0.0)),
            velocity: Arc::new(Mutex::new(0.0)),
            last_coupler_state: Arc::new(AtomicBool::new(false)),
            last_direction: Arc::new(Mutex::new(0.0)),
            last_update_time: Arc::new(Mutex::new(std::time::Instant::now())),
            door_speed,
        }
    }

    fn update(&self, coupler_state: bool) {
        // Calculate actual dt based on elapsed time
        let now = std::time::Instant::now();
        let mut last_time = self.last_update_time.lock().unwrap();
        let dt = now.duration_since(*last_time).as_secs_f64();
        *last_time = now;

        let last_state = self.last_coupler_state.load(Ordering::SeqCst);
        if coupler_state && !last_state {
            // Coupler just activated
            let pos = *self.position.lock().unwrap();
            let vel = *self.velocity.lock().unwrap();
            let last_dir = *self.last_direction.lock().unwrap();
            
            let new_vel = if vel == 0.0 {
                // Start moving with the calculated speed
                if pos <= 0.0 { self.door_speed } // Moving up from closed
                else if pos >= 1.0 { -self.door_speed } // Moving down from open
                else if last_dir > 0.0 { -self.door_speed } // Last moved up, now move down
                else if last_dir < 0.0 { self.door_speed } // Last moved down, now move up
                else { self.door_speed } // Default direction if no history
            } else {
                0.0 // Stop moving
            };
            *self.velocity.lock().unwrap() = new_vel;
            
            // If we're starting to move, record the direction
            if new_vel != 0.0 {
                *self.last_direction.lock().unwrap() = new_vel;
            }
        }
        self.last_coupler_state.store(coupler_state, Ordering::SeqCst);

        // Update position
        let mut pos = self.position.lock().unwrap();
        let vel = *self.velocity.lock().unwrap();
        *pos += vel * dt;

        // Clamp position and stop at limits
        if *pos <= 0.0 {
            *pos = 0.0;
            if vel < 0.0 {
                *self.velocity.lock().unwrap() = 0.0;
            }
        }
        if *pos >= 1.0 {
            *pos = 1.0;
            if vel > 0.0 {
                *self.velocity.lock().unwrap() = 0.0;
            }
        }
    }

    fn get_position(&self) -> f64 {
        *self.position.lock().unwrap()
    }
}

#[cfg(not(feature = "raspberry_pi"))]
pub fn create_pins(_close_pin: u8, _open_pin: u8, _coupler_pin: u8, poll_interval: Duration, expected_shut_time: Duration) -> Result<(MockInputPin, MockInputPin, MockOutputPin), Box<dyn std::error::Error>> {
    let simulation = DoorSimulation::new(expected_shut_time);
    // Initialize pins with correct states for a closed door:
    // - Close limit switch is pressed (LOW) when door is closed
    // - Open limit switch is not pressed (HIGH) when door is closed
    let close_pin = MockInputPin::new(false);  // LOW = pressed = door is closed
    let open_pin = MockInputPin::new(true);    // HIGH = not pressed
    let coupler = MockOutputPin::new();

    // Create thread-safe references
    let sim = simulation.clone();
    let close_pin_ref = close_pin.clone();
    let open_pin_ref = open_pin.clone();
    let coupler_ref = coupler.clone();

    // Spawn simulation thread
    thread::spawn(move || {
        loop {
            let coupler_state = coupler_ref.is_set_high();
            sim.update(coupler_state);
            
            let pos = sim.get_position();

            println!("Door position: {}", pos);

            // Update limit switches based on door position
            // LOW (false) when pressed, HIGH (true) when not pressed
            close_pin_ref.set_state(pos > 0.0);  // Close switch pressed (LOW) only when fully closed
            open_pin_ref.set_state(pos < 1.0);   // Open switch pressed (LOW) only when fully open
            
            thread::sleep(poll_interval);
        }
    });

    Ok((close_pin, open_pin, coupler))
}
