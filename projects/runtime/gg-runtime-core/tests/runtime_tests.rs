use gg_runtime_core::{EngineHost, FrameTime, InputEvents, DeltaTimer, FrameLimiter, ScriptEngine};use gg_ecs::World;use std::time::{Duration, Instant};

#[test]
fn test_engine_host_new() {
    let host = EngineHost::new();
    assert!(host.world().entities().is_empty());
}

#[test]
fn test_engine_host_world_access() {
    let mut host = EngineHost::new();
    let entity = host.world_mut().spawn().id();
    assert!(host.world().contains_entity(entity));
    assert_eq!(host.world().entities().len(), 1);
}

#[test]
fn test_engine_host_registry_access() {
    let host = EngineHost::new();
    assert!(!host.registry().is_registered("nonexistent"));
}

#[test]
fn test_frame_time_fields() {
    let ft = FrameTime {
        delta_seconds: 0.016,
        fixed_delta_seconds: 0.02,
        elapsed_seconds: 1.0,
    };
    assert!((ft.delta_seconds - 0.016).abs() < f32::EPSILON);
    assert!((ft.fixed_delta_seconds - 0.02).abs() < f32::EPSILON);
    assert!((ft.elapsed_seconds - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_input_events_empty() {
    let events = InputEvents { events: vec![] };
    assert!(events.events.is_empty());
}

#[test]
fn test_delta_timer_tick() {
    let mut timer = DeltaTimer::new();
    let delta = timer.tick();
    assert!(delta.as_secs_f32() >= 0.0);
}

#[test]
fn test_delta_timer_elapsed() {
    let mut timer = DeltaTimer::new();
    timer.tick();
    let elapsed = timer.elapsed();
    assert!(elapsed.as_secs() >= 0);
}

#[test]
fn test_delta_timer_delta_seconds() {
    let dur = Duration::from_millis(16);
    let secs = DeltaTimer::delta_seconds(dur);
    assert!((secs - 0.016).abs() < 0.001);
}

#[test]
fn test_frame_limiter_new() {
    let limiter = FrameLimiter::new(60);
    let fps = limiter.target_fps();
    assert!(fps >= 59 && fps <= 61);
}

#[test]
fn test_frame_limiter_from_fps() {
    let limiter = FrameLimiter::from_fps(30);
    let fps = limiter.target_fps();
    assert!(fps >= 29 && fps <= 31);
}

#[test]
fn test_frame_limiter_target_frame_time() {
    let limiter = FrameLimiter::new(60);
    let expected = Duration::from_secs_f64(1.0 / 60.0);
    assert!(limiter.target_frame_time() >= expected - Duration::from_micros(1));
    assert!(limiter.target_frame_time() <= expected + Duration::from_micros(1));
}

#[test]
fn test_script_engine_new() {
    let engine = ScriptEngine::new();
    assert!(!engine.has_script());
}

#[test]
fn test_input_events_with_keyboard() {
    use gg_core::platform::{InputEvent, KeyCode, KeyState};
    let events = InputEvents {
        events: vec![InputEvent::Keyboard { key: KeyCode::A, state: KeyState::Pressed }],
    };
    assert_eq!(events.events.len(), 1);
    match &events.events[0] {
        InputEvent::Keyboard { key, state } => {
            assert_eq!(*key, KeyCode::A);
            assert_eq!(*state, KeyState::Pressed);
        }
        _ => panic!("Expected Keyboard event"),
    }
}

#[test]
fn test_input_events_with_pointer() {
    use gg_core::platform::{InputEvent, PointerAction, PointerButton};
    let events = InputEvents {
        events: vec![InputEvent::Pointer {
            position: (100.0, 200.0),
            action: PointerAction::Down,
            button: Some(PointerButton::Left),
        }],
    };
    assert_eq!(events.events.len(), 1);
    match &events.events[0] {
        InputEvent::Pointer { position, action, button } => {
            assert_eq!(position.0, 100.0);
            assert_eq!(position.1, 200.0);
            assert_eq!(*action, PointerAction::Down);
            assert_eq!(*button, Some(PointerButton::Left));
        }
        _ => panic!("Expected Pointer event"),
    }
}

#[test]
fn test_input_events_replacement() {
    use gg_core::platform::{InputEvent, KeyCode, KeyState, PointerAction};
    let mut world = World::new();
    world.insert_resource(InputEvents {
        events: vec![InputEvent::Keyboard { key: KeyCode::A, state: KeyState::Pressed }],
    });
    assert_eq!(world.get_resource::<InputEvents>().unwrap().events.len(), 1);
    world.insert_resource(InputEvents {
        events: vec![
            InputEvent::Pointer {
                position: (50.0, 75.0),
                action: PointerAction::Move,
                button: None,
            },
        ],
    });
    let replaced = world.get_resource::<InputEvents>().unwrap();
    assert_eq!(replaced.events.len(), 1);
    match &replaced.events[0] {
        InputEvent::Pointer { .. } => {}
        _ => panic!("Expected only Pointer event after replacement"),
    }
}

#[test]
fn test_frame_time_default_fixed_delta() {
    let ft = FrameTime {
        delta_seconds: 0.016,
        fixed_delta_seconds: 0.0,
        elapsed_seconds: 0.0,
    };
    assert!(ft.fixed_delta_seconds.abs() < f32::EPSILON);
}

#[test]
fn test_frame_limiter_sleep_if_needed_no_sleep() {
    let limiter = FrameLimiter::new(60);
    let frame_elapsed = limiter.target_frame_time() + Duration::from_millis(5);
    limiter.sleep_if_needed(frame_elapsed);
}

#[test]
fn test_frame_limiter_30fps() {
    let limiter = FrameLimiter::new(30);
    let target = limiter.target_frame_time();
    let expected = Duration::from_secs_f64(1.0 / 30.0);
    let diff = if target > expected { target - expected } else { expected - target };
    assert!(diff < Duration::from_micros(100));
}
