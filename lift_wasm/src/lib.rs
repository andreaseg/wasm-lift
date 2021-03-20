mod utils;

#[macro_use]
extern crate lazy_static;

use wasm_bindgen::prelude::*;

use lift::*;

use std::sync::Mutex;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Lift {
    position: Position,
    velocity: Velocity,
    floors_to_stop_at: Vec<Floor>,
    is_emergency_stop_activated: bool,
    is_stopped: bool
}

impl Lift {
    fn remove_current_floor(&mut self) {
        let current_floor = self.position.round() as Floor;
        if let Some(index) = self
            .floors_to_stop_at
            .iter()
            .enumerate()
            .find(|(_, e)| **e == current_floor)
            .map(|(i, _)| i)
        {
            self.floors_to_stop_at.remove(index);
        };
    }
}

impl LiftSensors for Lift {
    fn current_floor(&self) -> Position {
        self.position
    }

    fn current_velocity(&self) -> Velocity {
        self.velocity
    }

    fn floors_to_stop_at(&self) -> &[Floor] {
        &self.floors_to_stop_at
    }

    fn is_emergency_stop_activated(&self) -> bool {
        self.is_emergency_stop_activated
    }
}

#[wasm_bindgen]
pub struct SimulationResult {
    pub position: Position,
    pub is_stopped: bool
}

impl From<&Lift> for SimulationResult {
    fn from(lift: &Lift) -> SimulationResult {
        SimulationResult {
            position: lift.position,
            is_stopped: lift.is_stopped
        }
    }
}

lazy_static! {
    static ref LIFT: Mutex<Lift> = Mutex::new(Lift {
        position: 0.0,
        velocity: 0.0,
        floors_to_stop_at: Vec::new(),
        is_emergency_stop_activated: false,
        is_stopped: false
    });
}

const VELOCITY: Velocity = 1.0;

lazy_static! {
    static ref CONTROLLER: Mutex<LiftController> = Mutex::new(LiftController::new(VELOCITY, 0.01, 0.01));
}

#[wasm_bindgen]
pub fn emergency_stop(status: bool) {
    let mut lift = LIFT.lock().unwrap();
    lift.is_emergency_stop_activated = status
}

#[wasm_bindgen]
pub fn stop_lift_at_floor(floor: Floor) {
    let mut lift = LIFT.lock().unwrap();
    lift.floors_to_stop_at.push(floor);
}

/// Step the simulation by the time as specified in 'time_step'
/// Returns true if the lift is still moving and false if its stopped
#[wasm_bindgen]
pub fn step_simulation(time_step: f32) -> SimulationResult {
    let mut lift = LIFT.lock().unwrap();
    let mut controller = CONTROLLER.lock().unwrap();
    let action = controller.poll(&*lift, time_step);
    if action.is_stopped_at_current_floor {
        lift.remove_current_floor();
        lift.is_stopped = true;
        return (&*lift).into();
    }
    lift.position += action.target_velocity * time_step;
    lift.velocity = action.target_velocity;

    lift.is_stopped = false;
    (&*lift).into()
}

#[wasm_bindgen]
pub fn last_simulation_result() -> SimulationResult {
    let lift = LIFT.lock().unwrap();
    (&*lift).into()
}

#[wasm_bindgen]
pub fn time_to_floor(floor: Floor, average_stop: f32) -> Option<f32> {
    let lift = LIFT.lock().unwrap();
    let controller = CONTROLLER.lock().unwrap();
    controller.time_to_floor(&*lift, floor, average_stop)
}