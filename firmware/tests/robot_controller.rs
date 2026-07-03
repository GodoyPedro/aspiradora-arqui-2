use robot_vacuum_firmware::domain::{
    AutoNavigationPhase, CleaningMode, ContactType, ManualDirection, RobotCommand, RobotState,
    WallSide,
};
use robot_vacuum_firmware::simulation::{create_simulation_controller, SimulationConfig};

#[test]
fn start_cleaning_from_standby() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    let status = controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");

    assert_eq!(status.state, RobotState::Cleaning);
    assert!(status.suction_enabled);
    assert!(status.brushes_enabled);
}

#[test]
fn stop_cleaning_from_cleaning() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");

    let status = controller
        .handle_command(RobotCommand::StopCleaning)
        .expect("stop cleaning should succeed");

    assert_eq!(status.state, RobotState::Standby);
    assert_eq!(status.left_wheel_speed, 0);
    assert_eq!(status.right_wheel_speed, 0);
    assert!(!status.suction_enabled);
    assert!(!status.brushes_enabled);
}

#[test]
fn pause_cleaning_from_cleaning() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");

    let status = controller
        .handle_command(RobotCommand::PauseCleaning)
        .expect("pause cleaning should succeed");

    assert_eq!(status.state, RobotState::Paused);
    assert_eq!(status.left_wheel_speed, 0);
    assert_eq!(status.right_wheel_speed, 0);
}

#[test]
fn manual_movement_changes_wheel_speeds() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    let status = controller
        .handle_command(RobotCommand::ManualMove {
            direction: ManualDirection::Left,
            speed: 42,
            duration_ms: 1_000,
        })
        .expect("manual move should succeed");

    assert_eq!(status.state, RobotState::ManualControl);
    assert_eq!(status.cleaning_mode, CleaningMode::Manual);
    assert_eq!(status.left_wheel_speed, -42);
    assert_eq!(status.right_wheel_speed, 42);
}

#[test]
fn manual_move_expires_after_simulated_time_advance() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::ManualMove {
            direction: ManualDirection::Forward,
            speed: 45,
            duration_ms: 250,
        })
        .expect("manual move should succeed");

    controller.clock_mut().advance_ms(300);
    let status = controller.tick();

    assert_eq!(status.state, RobotState::Standby);
    assert_eq!(status.left_wheel_speed, 0);
    assert_eq!(status.right_wheel_speed, 0);
    assert_eq!(status.cleaning_mode, CleaningMode::Auto);
}

#[test]
fn status_is_readable_during_error() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().drop_off_detected = true;
    controller.tick();

    let status = controller.current_status();

    assert_eq!(status.state, RobotState::Error);
    assert_eq!(
        status.current_error.map(|err| err.to_string()),
        Some("DROP_OFF_DETECTED".into())
    );
}

#[test]
fn start_cleaning_enters_room_crossing() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Cleaning);
    assert_eq!(
        status.auto_navigation_phase,
        Some(AutoNavigationPhase::RoomCrossing)
    );
    assert_eq!(status.left_wheel_speed, 36);
    assert_eq!(status.right_wheel_speed, 36);
}

#[test]
fn proximity_wall_contact_enters_wall_recovery() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().proximity_contact = true;
    controller.sensor_reader_mut().bumper_pressed = true;
    controller.sensor_reader_mut().contact_type = Some(ContactType::Wall);
    controller.sensor_reader_mut().wall_side = Some(WallSide::Left);

    let backup = controller.tick();
    assert_eq!(
        backup.auto_navigation_phase,
        Some(AutoNavigationPhase::WallBackup)
    );
    assert_eq!(backup.left_wheel_speed, -24);
    assert_eq!(backup.right_wheel_speed, -24);

    controller.clock_mut().advance_ms(900);
    controller.sensor_reader_mut().proximity_contact = false;
    controller.sensor_reader_mut().bumper_pressed = false;
    let align = controller.tick();
    assert_eq!(
        align.auto_navigation_phase,
        Some(AutoNavigationPhase::WallTurnAway)
    );

    controller.clock_mut().advance_ms(15_000);
    controller.tick();
    let follow = controller.tick();
    assert_eq!(
        follow.auto_navigation_phase,
        Some(AutoNavigationPhase::WallExitForward)
    );
    assert_eq!(follow.left_wheel_speed, 36);
    assert_eq!(follow.right_wheel_speed, 36);
}

#[test]
fn clearance_turn_clear_holds_until_sensor_clears() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().proximity_contact = true;
    controller.sensor_reader_mut().bumper_pressed = true;
    controller.sensor_reader_mut().contact_type = Some(ContactType::Obstacle);
    controller.sensor_reader_mut().wall_side = None;
    controller.tick();
    controller.clock_mut().advance_ms(900);
    controller.tick();
    controller.clock_mut().advance_ms(60_000);
    controller.tick();

    let status = controller.tick();
    assert_eq!(
        status.auto_navigation_phase,
        Some(AutoNavigationPhase::ClearanceTurnClear)
    );
    assert_eq!(status.left_wheel_speed.abs(), 6);
    assert_eq!(status.right_wheel_speed.abs(), 6);
    assert_eq!(status.left_wheel_speed, -status.right_wheel_speed);
}

#[test]
fn wall_contact_uses_wall_recovery_phases() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().proximity_contact = true;
    controller.sensor_reader_mut().bumper_pressed = true;
    controller.sensor_reader_mut().contact_type = Some(ContactType::Wall);
    controller.sensor_reader_mut().wall_side = Some(WallSide::Left);
    controller.tick();
    controller.clock_mut().advance_ms(900);
    controller.tick();
    controller.tick();

    controller.sensor_reader_mut().proximity_contact = true;
    controller.sensor_reader_mut().bumper_pressed = true;
    controller.sensor_reader_mut().contact_type = Some(ContactType::Wall);
    let corner_backup = controller.tick();
    assert_eq!(
        corner_backup.auto_navigation_phase,
        Some(AutoNavigationPhase::WallTurnAway)
    );
    assert_eq!(corner_backup.left_wheel_speed.abs(), 6);
    assert_eq!(corner_backup.right_wheel_speed.abs(), 6);
    assert_eq!(
        corner_backup.left_wheel_speed,
        -corner_backup.right_wheel_speed
    );

    controller.sensor_reader_mut().proximity_contact = false;
    controller.sensor_reader_mut().bumper_pressed = false;
    let corner_turn = controller.tick();
    assert_eq!(
        corner_turn.auto_navigation_phase,
        Some(AutoNavigationPhase::WallTurnAway)
    );

    controller.clock_mut().advance_ms(15_000);
    controller.tick();
    let corner_exit = controller.tick();
    assert_eq!(
        corner_exit.auto_navigation_phase,
        Some(AutoNavigationPhase::WallExitForward)
    );

    controller.clock_mut().advance_ms(1_600);
    let corner_exit = controller.tick();
    assert_eq!(
        corner_exit.auto_navigation_phase,
        Some(AutoNavigationPhase::RoomCrossing)
    );
    assert_eq!(corner_exit.left_wheel_speed, 36);
    assert_eq!(corner_exit.right_wheel_speed, 36);
}

#[test]
fn obstacle_contact_enters_clearance_escape() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().proximity_contact = true;
    controller.sensor_reader_mut().bumper_pressed = true;
    controller.sensor_reader_mut().contact_type = Some(ContactType::Obstacle);

    let backup = controller.tick();
    assert_eq!(
        backup.auto_navigation_phase,
        Some(AutoNavigationPhase::ClearanceBackup)
    );
    assert_eq!(backup.left_wheel_speed, -24);
    assert_eq!(backup.right_wheel_speed, -24);

    controller.clock_mut().advance_ms(900);
    controller.sensor_reader_mut().proximity_contact = false;
    controller.sensor_reader_mut().bumper_pressed = false;
    let turn_away = controller.tick();
    assert_eq!(
        turn_away.auto_navigation_phase,
        Some(AutoNavigationPhase::ClearanceTurnClear)
    );

    controller.tick();
    controller.tick();
    let escape = controller.tick();
    assert_eq!(
        escape.auto_navigation_phase,
        Some(AutoNavigationPhase::ClearanceExitForward)
    );
    assert_eq!(escape.left_wheel_speed, 36);
    assert_eq!(escape.right_wheel_speed, 36);
}

#[test]
fn obstacle_detected_alone_does_not_trigger_escape() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().obstacle_detected = true;
    controller.sensor_reader_mut().proximity_contact = false;
    controller.sensor_reader_mut().bumper_pressed = false;

    let status = controller.tick();

    assert_eq!(
        status.auto_navigation_phase,
        Some(AutoNavigationPhase::RoomCrossing)
    );
    assert_eq!(status.left_wheel_speed, 36);
    assert_eq!(status.right_wheel_speed, 36);
}

#[test]
fn low_battery_triggers_returning_to_dock() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.battery_driver_mut().set_percentage(15);

    let status = controller.tick();

    assert_eq!(status.state, RobotState::ReturningToDock);
    assert_eq!(status.left_wheel_speed, 30);
    assert_eq!(status.right_wheel_speed, 30);
}

#[test]
fn dock_detection_transitions_to_charging() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.battery_driver_mut().set_percentage(15);
    controller.tick();
    controller.docking_driver_mut().set_dock_detected(true);

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Charging);
    assert!(status.is_charging);
}

#[test]
fn drop_off_transitions_to_error_and_stops_actuators() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().drop_off_detected = true;

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Error);
    assert_eq!(status.left_wheel_speed, 0);
    assert!(!status.suction_enabled);
    assert!(!status.brushes_enabled);
}

#[test]
fn wheel_stuck_transitions_to_error_and_stops_actuators() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().wheel_stuck = true;

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Error);
    assert_eq!(
        status.current_error.map(|err| err.to_string()),
        Some("WHEEL_STUCK".into())
    );
    assert_eq!(status.left_wheel_speed, 0);
}

#[test]
fn brush_stuck_transitions_to_error_and_stops_actuators() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().brush_stuck = true;

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Error);
    assert_eq!(
        status.current_error.map(|err| err.to_string()),
        Some("BRUSH_STUCK".into())
    );
    assert!(!status.suction_enabled);
}

#[test]
fn top_cover_open_transitions_to_error_and_stops_actuators() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().top_cover_open = true;

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Error);
    assert_eq!(
        status.current_error.map(|err| err.to_string()),
        Some("TOP_COVER_OPEN".into())
    );
    assert!(!status.brushes_enabled);
}

#[test]
fn dust_container_full_transitions_to_error_and_stops_actuators() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().dust_container_full = true;

    let status = controller.tick();

    assert_eq!(status.state, RobotState::Error);
    assert_eq!(
        status.current_error.map(|err| err.to_string()),
        Some("DUST_CONTAINER_FULL".into())
    );
    assert_eq!(status.left_wheel_speed, 0);
    assert_eq!(status.right_wheel_speed, 0);
    assert!(!status.suction_enabled);
    assert!(!status.brushes_enabled);
}

#[test]
fn clear_error_is_rejected_while_dust_container_is_still_full() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().dust_container_full = true;
    controller.tick();

    let error = controller
        .handle_command(RobotCommand::ClearError)
        .expect_err("clear error should fail while dust container is still full");

    assert_eq!(error.code, "DUST_CONTAINER_FULL");
    assert_eq!(controller.current_status().state, RobotState::Error);
}

#[test]
fn clear_error_returns_to_standby_when_safe() {
    let mut controller = create_simulation_controller(SimulationConfig::default());
    controller
        .handle_command(RobotCommand::StartCleaning)
        .expect("start cleaning should succeed");
    controller.sensor_reader_mut().drop_off_detected = true;
    controller.tick();
    controller.sensor_reader_mut().drop_off_detected = false;

    let status = controller
        .handle_command(RobotCommand::ClearError)
        .expect("clear error should succeed");

    assert_eq!(status.state, RobotState::Standby);
    assert!(status.current_error.is_none());
}

#[test]
fn invalid_command_state_combinations_are_rejected_safely() {
    let mut controller = create_simulation_controller(SimulationConfig::default());

    let error = controller
        .handle_command(RobotCommand::PauseCleaning)
        .expect_err("pause from standby should fail");

    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
    assert_eq!(controller.current_status().state, RobotState::Standby);
}

#[test]
fn unsupported_mode_is_rejected_safely() {
    let mut controller = create_simulation_controller(SimulationConfig::default());

    let error = controller
        .handle_command(RobotCommand::SetCleaningMode(CleaningMode::Spot))
        .expect_err("spot mode should be unsupported");

    assert_eq!(error.code, "UNSUPPORTED_MODE");
    assert_eq!(
        controller.current_status().cleaning_mode,
        CleaningMode::Auto
    );
}

#[test]
fn manual_move_zero_speed_for_movement_direction_is_rejected() {
    let mut controller = create_simulation_controller(SimulationConfig::default());

    let error = controller
        .handle_command(RobotCommand::ManualMove {
            direction: ManualDirection::Forward,
            speed: 0,
            duration_ms: 1_000,
        })
        .expect_err("zero speed forward move should fail");

    assert_eq!(error.code, "INVALID_SPEED");
    assert_eq!(controller.current_status().state, RobotState::Standby);
}

#[test]
fn manual_move_stop_with_zero_speed_is_allowed() {
    let mut controller = create_simulation_controller(SimulationConfig::default());

    let status = controller
        .handle_command(RobotCommand::ManualMove {
            direction: ManualDirection::Stop,
            speed: 0,
            duration_ms: 1_000,
        })
        .expect("stop move with zero speed should succeed");

    assert_eq!(status.state, RobotState::ManualControl);
    assert_eq!(status.left_wheel_speed, 0);
    assert_eq!(status.right_wheel_speed, 0);
}
