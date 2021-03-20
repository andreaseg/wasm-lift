#![no_std]

#[allow(unused_imports)]
use micromath::F32Ext;

/// Position in normalised units, the distance between two floors in the elevator is '1.0'
pub type Position = f32;

/// Velocity in normalised units, the velocity is given as 'floors / second'
pub type Velocity = f32;

/// Signed integer representaiton of actual floor
pub type Floor = i32;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Neutral,
}

/// The properties associated with a given lift
pub struct LiftController {
    /// Prefered target velocity for the elevator
    prefered_velocity: Velocity,

    /// Floor leeway, if a elevator position differs from a floor position by less than the floor_leeway and the elevator
    /// is stopped it is not considered safely stopped
    floor_leeway: Position,

    /// The allowed sensor difference in velocity. If a velocity is below this it is considered zero
    velocity_epsilon: Velocity,

    /// The current direction of the elevator
    direction: Direction,
}

/// Trait to be implemented by a Lift implementation representing different sensors.
/// Units must be normalized such that the distance unit is un floor-distance and
/// the time unit is in seconds
pub trait LiftSensors {
    /// Sensor reading of the current floor
    fn current_floor(&self) -> Position;

    /// Sensor reading of the current velocity
    fn current_velocity(&self) -> Velocity;

    /// A list of floors to stop at
    fn floors_to_stop_at(&self) -> &[Floor];

    /// If the emergency stop has been activated
    fn is_emergency_stop_activated(&self) -> bool;
}

/// Recommended action from the LiftController
pub struct Action {
    /// Velocity to target
    pub target_velocity: Velocity,

    /// If the controller considers the lift to be stopped at the current floor
    /// When this is set to true it signals that the lift has reached one of its target floors,
    /// or that it has been stopped at a floor due to the emergency stop.
    ///
    /// Embarkment, door control and such actions should now be possible.
    ///
    /// In a loop this parameter may be queried for in order to change the state from 'moving'
    /// to 'stopped'.
    pub is_stopped_at_current_floor: bool,
}

impl LiftController {
    /// Create a new LiftController
    /// The associated LiftProperties must also be provided to properly control the lift
    pub const fn new(
        prefered_velocity: Velocity,
        floor_leeway: Position,
        velocity_epsilon: Velocity,
    ) -> Self {
        LiftController {
            prefered_velocity,
            floor_leeway,
            velocity_epsilon,
            direction: Direction::Neutral,
        }
    }

    /// From sensor data, poll for the next action to perform
    pub fn poll(&mut self, sensors: &dyn LiftSensors, time_step: f32) -> Action {
        let is_stopped = sensors.current_velocity().abs() < self.velocity_epsilon;
        let can_stop_at_floor = self.can_stop_at_floor(sensors);
        let is_stopped_at_current_floor = is_stopped && can_stop_at_floor.is_some();

        // If the emergency step sensor is active this should take the absolutely highest proprity
        if sensors.is_emergency_stop_activated() {
            return Action {
                target_velocity: 0.0,
                is_stopped_at_current_floor,
            };
        }

        if let (direction, Some(next_target_floor)) = next_target_floor(
            &self.direction,
            sensors.current_floor(),
            self.floor_leeway,
            sensors.floors_to_stop_at(),
        ) {
            // A target floor is set

            /*
            If a non-neutral direction is given change the direction.
            In this case we treat Neutral as 'go the same direction'
            */
            if direction != Direction::Neutral {
                self.direction = direction;
            }

            /*
            We take special consideration here when calculating the target velocity.
            If the time_step is too high we need to make sure we don't overshoot the floor.
            */
            let signed_distance = next_target_floor as f32 - sensors.current_floor();

            let exact_target_velocity = (signed_distance / time_step).abs();

            let target_velocity =
                f32::min(self.prefered_velocity, exact_target_velocity).copysign(signed_distance);

            Action {
                target_velocity,
                is_stopped_at_current_floor: false,
            }
        } else {
            // No target floor is set, we can simply wait at the current floor
            Action {
                target_velocity: 0.0,
                is_stopped_at_current_floor: true,
            }
        }
    }

    /// Check if it is possible to stop currently
    /// Returns Some(Floor) if it is possible to stop, and None if it is impossible
    fn can_stop_at_floor(&self, sensors: &dyn LiftSensors) -> Option<Floor> {
        let nearest_floor = sensors.current_floor().round();
        let floor_distance = (sensors.current_floor() - nearest_floor).abs();

        if floor_distance < self.floor_leeway {
            Some(nearest_floor as Floor)
        } else {
            None
        }
    }

    pub fn time_to_floor(
        &self,
        sensors: &dyn LiftSensors,
        floor: Floor,
        average_stop: f32,
    ) -> Option<f32> {
        let current_floor = sensors.current_floor();
        let speed = sensors.current_velocity().abs();

        if speed < self.velocity_epsilon {
            return None;
        }

        let floors = sensors.floors_to_stop_at();

        let target = floor as f32;

        let highest_floor: f32 = floors.iter().max().map(|f| *f as f32).unwrap_or(0f32);
        let lowest_floor: f32 = floors.iter().min().map(|f| *f as f32).unwrap_or(0f32);

        match (&self.direction, target > current_floor) {
            (Direction::Neutral, _) => None,
            (Direction::Up, true) => {
                let above: f32 = floors
                    .iter()
                    .copied()
                    .map(|f| f as f32)
                    .filter(|f| *f > current_floor && *f < target)
                    .count() as f32 * average_stop;

                let distance = target - current_floor;

                Some(above + distance / speed)
            }
            (Direction::Up, false) => {
                let above: f32 = floors
                    .iter()
                    .copied()
                    .map(|f| f as f32)
                    .filter(|f| *f > current_floor && *f < highest_floor)
                    .count() as f32 * average_stop;

                let below: f32 = floors
                    .iter()
                    .copied()
                    .map(|f| f as f32)
                    .filter(|f| *f < current_floor && *f > target)
                    .count() as f32 * average_stop;

                let distance = highest_floor - current_floor + highest_floor - target;

                Some(above + below + distance / speed)
            }
            (Direction::Down, true) => {
                let above: f32 = floors
                    .iter()
                    .copied()
                    .map(|f| f as f32)
                    .filter(|f| *f > current_floor && *f < target)
                    .count() as f32 * average_stop;

                let below: f32 = floors
                    .iter()
                    .copied()
                    .map(|f| f as f32)
                    .filter(|f| *f < current_floor && *f > lowest_floor)
                    .count() as f32 * average_stop;

                let distance = current_floor - lowest_floor + target - lowest_floor;

                Some(above + below + distance / speed)
            }
            (Direction::Down, false) => {
                let below: f32 = floors
                    .iter()
                    .copied()
                    .map(|f| f as f32 * average_stop)
                    .filter(|f| *f < current_floor && *f > target)
                    .sum();

                let distance = current_floor - target;

                Some(below + distance / speed)
            }
        }
    }
}

/// Find the next target floor and the direction to it
fn next_target_floor(
    direction: &Direction,
    current_floor: Position,
    floor_leeway: Position,
    floors: &[Floor],
) -> (Direction, Option<Floor>) {
    /*
    Find the nearest floor. If there is no floor in the current direction, try to look in the other direction.
    This strategy of priorizing the current direction is important to reduce (acutally make bounds on)
    the worst case pickup time of for any passengers. With this strategy we can ensure that for a building
    that is N stories tall the lift will make no more than (N - 1) stops before picking up a passenger,
    and likewise will make at most (N - 1) stops before dropping them off.
    */
    let target_floor = match direction {
        Direction::Up => nearest_floor_above(current_floor.round() as i32, floors)
            .or_else(|| nearest_floor_below(current_floor.round() as i32, floors)),
        Direction::Down => nearest_floor_below(current_floor.round() as i32, floors)
            .or_else(|| nearest_floor_above(current_floor.round() as i32, floors)),
        Direction::Neutral => nearest_floor(current_floor.round() as i32, floors),
    }
    /*
    We filter away the current floor from the consideration, this may not be strictly
    necessary, but since we have made the Lift implementation generic we can't make
    any assumptions about when the floor-list is cleared
    */
    .filter(|floor| (*floor as f32 - current_floor).abs() > floor_leeway);

    match target_floor {
        Some(target_floor) => {
            let direction_delta = target_floor - current_floor.round() as i32;
            let direction = match direction_delta {
                1..=i32::MAX => Direction::Up,
                0 => Direction::Neutral,
                i32::MIN..=-1 => Direction::Down,
            };

            (direction, Some(target_floor))
        }
        None => (Direction::Neutral, None),
    }
}

fn nearest_floor_above(current_floor: Floor, floors_to_stop_at: &[Floor]) -> Option<Floor> {
    floors_to_stop_at
        .iter()
        .filter(|floor| **floor >= current_floor)
        .min_by_key(|floor| **floor - current_floor)
        .copied()
}

fn nearest_floor_below(current_floor: Floor, floors_to_stop_at: &[Floor]) -> Option<Floor> {
    floors_to_stop_at
        .iter()
        .filter(|floor| **floor <= current_floor)
        .min_by_key(|floor| current_floor - **floor)
        .copied()
}

fn nearest_floor(current_floor: Floor, floors_to_stop_at: &[Floor]) -> Option<Floor> {
    floors_to_stop_at
        .iter()
        .min_by_key(|floor| (current_floor - **floor).abs())
        .copied()
}

#[cfg(test)]
mod tests {

    extern crate std;

    use super::*;
    use std::{fmt, println, vec::Vec};

    #[derive(Debug)]
    struct TestLift {
        position: Position,
        velocity: Velocity,
        floors_to_stop_at: Vec<Floor>,
        is_emergency_stop_activated: bool,
    }

    impl fmt::Debug for LiftController {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("LiftController")
                .field("prefered_velocity", &self.prefered_velocity)
                .field("floor_leeway", &self.floor_leeway)
                .field("velocity_epsilon", &self.velocity_epsilon)
                .field("direction", &self.direction)
                .finish()
        }
    }

    impl fmt::Debug for Direction {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self {
                Direction::Down => f.write_str("Down"),
                Direction::Up => f.write_str("Up"),
                Direction::Neutral => f.write_str("Neutral"),
            }
        }
    }

    /// Changes the scale of a decimal number to have the given number of decimals
    /// Useful is small floating point errors lead to failing tests
    fn scale(number: f32, decimals: i32) -> f32 {
        let factor = 10f32.powi(decimals);
        (number * factor).round() / factor
    }

    fn find<T: PartialEq>(elem: &T, slice: &[T]) -> Option<usize> {
        slice
            .iter()
            .enumerate()
            .find(|(_, e)| *e == elem)
            .map(|(i, _)| i)
    }

    impl TestLift {
        fn new() -> Self {
            TestLift {
                position: 0.0,
                velocity: 0.0,
                floors_to_stop_at: Vec::new(),
                is_emergency_stop_activated: false,
            }
        }

        fn stop_at_floor(&mut self, floor: Floor) {
            if let None = find(&floor, &self.floors_to_stop_at()) {
                self.floors_to_stop_at.push(floor);
            }
        }

        fn remove_floor_from_panel(&mut self, floor: Floor) {
            if let Some(i) = find(&floor, &self.floors_to_stop_at()) {
                self.floors_to_stop_at.remove(i);
            }
        }

        fn accept_action(&mut self, action: Action, time_step: f32) {
            self.position += action.target_velocity * time_step;
            self.velocity = action.target_velocity;
            if action.is_stopped_at_current_floor {
                self.remove_floor_from_panel(self.position.round() as i32);
            }
        }
    }

    impl LiftSensors for TestLift {
        fn current_floor(&self) -> Position {
            self.position
        }

        fn current_velocity(&self) -> Velocity {
            self.velocity
        }

        fn floors_to_stop_at(&self) -> &[Floor] {
            self.floors_to_stop_at.as_slice()
        }

        fn is_emergency_stop_activated(&self) -> bool {
            self.is_emergency_stop_activated
        }
    }

    #[test]
    fn go_to_tenth_floor() {
        let mut lift = TestLift::new();

        let mut controller = LiftController::new(0.5, 0.001, 0.001);

        let time = 20f32;
        let time_step = 0.1f32;
        let steps = (time / time_step) as i32;

        lift.stop_at_floor(10);

        for step in 0..steps {
            let action = controller.poll(&lift, time_step);
            lift.accept_action(action, time_step);
            println!(
                "{: >2.1} {: >2.2} {: >2.2}",
                (step as f32 * time_step),
                lift.position,
                lift.velocity
            );
        }

        assert_eq!(lift.current_floor(), 10f32);
    }

    #[test]
    fn switch_direction() {
        let mut lift = TestLift::new();

        let mut controller = LiftController::new(0.5, 0.001, 0.001);
        let time = 20f32;
        let time_step = 0.1f32;
        let steps = (time / time_step) as i32;

        lift.stop_at_floor(5);
        for _ in 0..steps {
            let action = controller.poll(&lift, time_step);
            if action.is_stopped_at_current_floor {
                println!("Goal is floor 5:\n{:#?}\n{:#?}", lift, controller);
                assert_eq!(5.0, lift.current_floor());
                lift.remove_floor_from_panel(5);
                break;
            }
            lift.accept_action(action, time_step);
        }

        lift.stop_at_floor(0);
        lift.stop_at_floor(1);
        lift.stop_at_floor(10);

        for _ in 0..steps {
            let action = controller.poll(&lift, time_step);
            if action.is_stopped_at_current_floor {
                println!("Goal is floor 10:\n{:#?}\n{:#?}", lift, controller);
                assert_eq!(10.0, lift.current_floor());
                lift.remove_floor_from_panel(10);
                break;
            }
            lift.accept_action(action, time_step);
        }

        for _ in 0..steps {
            let action = controller.poll(&lift, time_step);
            if action.is_stopped_at_current_floor {
                println!("Goal is floor 1:\n{:#?}\n{:#?}", lift, controller);
                assert_eq!(1.0, lift.current_floor());
                lift.remove_floor_from_panel(1);
                break;
            }
            lift.accept_action(action, time_step);
        }

        for _ in 0..steps {
            let action = controller.poll(&lift, time_step);
            if action.is_stopped_at_current_floor {
                println!("Goal is floor 0:\n{:#?}\n{:#?}", lift, controller);
                assert_eq!(0.0, lift.current_floor());
                lift.remove_floor_from_panel(0);
                break;
            }
            lift.accept_action(action, time_step);
        }
    }

    #[test]
    fn emergency_stop() {
        let mut lift = TestLift::new();

        let velocity = 0.5f32;
        let mut controller = LiftController::new(velocity, 0.001, 0.001);
        let time = 60f32;
        let time_step = 0.1f32;
        let steps = (time / time_step) as i32;
        let time_to_emergency = 20f32;

        lift.stop_at_floor(9000);
        for i in 0..steps {
            if i as f32 * time_step >= time_to_emergency {
                lift.is_emergency_stop_activated = true
            }

            let action = controller.poll(&lift, time_step);
            lift.accept_action(action, time_step);
        }

        assert_eq!(velocity * time_to_emergency, scale(lift.current_floor(), 4));
    }
}
