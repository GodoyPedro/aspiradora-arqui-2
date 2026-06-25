use crate::domain::{DomainError, RobotState};

#[derive(Debug)]
pub struct RobotStateMachine {
    state: RobotState,
}

impl Default for RobotStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl RobotStateMachine {
    pub fn new() -> Self {
        Self {
            state: RobotState::Standby,
        }
    }

    pub fn state(&self) -> RobotState {
        self.state
    }

    pub fn transition_to(&mut self, next: RobotState) -> Result<(), DomainError> {
        if self.state == next {
            return Ok(());
        }

        let allowed = matches!(
            (self.state, next),
            (RobotState::Off, RobotState::Standby)
                | (RobotState::Standby, RobotState::Cleaning)
                | (RobotState::Standby, RobotState::ManualControl)
                | (RobotState::Standby, RobotState::ReturningToDock)
                | (RobotState::Standby, RobotState::Error)
                | (RobotState::Cleaning, RobotState::Paused)
                | (RobotState::Cleaning, RobotState::ReturningToDock)
                | (RobotState::Cleaning, RobotState::Standby)
                | (RobotState::Cleaning, RobotState::Error)
                | (RobotState::Paused, RobotState::Cleaning)
                | (RobotState::Paused, RobotState::ManualControl)
                | (RobotState::Paused, RobotState::ReturningToDock)
                | (RobotState::Paused, RobotState::Standby)
                | (RobotState::Paused, RobotState::Error)
                | (RobotState::ManualControl, RobotState::Standby)
                | (RobotState::ManualControl, RobotState::Paused)
                | (RobotState::ManualControl, RobotState::ReturningToDock)
                | (RobotState::ManualControl, RobotState::Error)
                | (RobotState::ReturningToDock, RobotState::Charging)
                | (RobotState::ReturningToDock, RobotState::Standby)
                | (RobotState::ReturningToDock, RobotState::Error)
                | (RobotState::Charging, RobotState::Standby)
                | (RobotState::Charging, RobotState::Error)
                | (RobotState::Error, RobotState::Standby)
        );

        if !allowed {
            return Err(DomainError::conflict(
                "INVALID_STATE_TRANSITION",
                format!("Cannot transition from {} to {}", self.state, next),
                self.state,
            ));
        }

        self.state = next;
        Ok(())
    }
}
