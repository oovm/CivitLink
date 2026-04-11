use gg_ecs::World;
use gg_runtime_core::{
    StageScheduler,
    scheduler::{Stage, StageScheduler},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[test]
fn test_stage_scheduler_new() {
    let scheduler = StageScheduler::new();
    assert!(!scheduler.is_startup_executed());
    assert_eq!(scheduler.system_count(Stage::Update), 0);
    assert_eq!(scheduler.system_count(Stage::Startup), 0);
}

#[test]
fn test_stage_scheduler_tick_empty() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();
    let result = scheduler.tick(&mut world, Duration::from_secs_f32(1.0 / 60.0));
    assert!(result.is_ok());
    assert!(scheduler.is_startup_executed());
}

#[test]
fn test_stage_scheduler_add_system() {
    let mut scheduler = StageScheduler::new();
    scheduler.add_system("test_system", Box::new(|_world| Ok(())));
    assert_eq!(scheduler.system_count(Stage::Update), 1);
}

#[test]
fn test_stage_scheduler_add_system_to_stage() {
    let mut scheduler = StageScheduler::new();
    scheduler.add_system_to_stage("startup_sys", Box::new(|_world| Ok(())), Stage::Startup);
    assert_eq!(scheduler.system_count(Stage::Startup), 1);
    assert_eq!(scheduler.system_count(Stage::Update), 0);
}

#[test]
fn test_stage_scheduler_tick_runs_systems() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();
    world.spawn().id();

    scheduler.add_system_to_stage(
        "spawn_system",
        Box::new(move |world: &mut World| {
            world.spawn();
            Ok(())
        }),
        Stage::Update,
    );

    let result = scheduler.tick(&mut world, Duration::from_secs_f32(1.0 / 60.0));
    assert!(result.is_ok());
    assert_eq!(world.entities().len(), 2);
}

#[test]
fn test_stage_scheduler_startup_runs_once() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    scheduler.add_system_to_stage(
        "startup_sys",
        Box::new(|world: &mut World| {
            world.spawn();
            Ok(())
        }),
        Stage::Startup,
    );

    scheduler.add_system_to_stage(
        "update_sys",
        Box::new(|world: &mut World| {
            world.spawn();
            Ok(())
        }),
        Stage::Update,
    );

    scheduler.tick(&mut world, Duration::from_secs_f32(1.0 / 60.0)).unwrap();
    assert!(scheduler.is_startup_executed());

    let count_after_first = world.entities().len();

    scheduler.tick(&mut world, Duration::from_secs_f32(1.0 / 60.0)).unwrap();
    let count_after_second = world.entities().len();

    assert_eq!(count_after_second - count_after_first, 1);
}

#[test]
fn test_stage_scheduler_stop() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    scheduler.add_system_to_stage(
        "exit_sys",
        Box::new(|world: &mut World| {
            world.spawn();
            Ok(())
        }),
        Stage::Exit,
    );

    let result = scheduler.stop(&mut world);
    assert!(result.is_ok());
    assert_eq!(world.entities().len(), 1);
}

#[test]
fn test_stage_scheduler_system_ordering_after() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    scheduler.add_system("first", Box::new(|_world| Ok(())));
    scheduler.add_system("second", Box::new(|_world| Ok(()))).after("first");

    let result = scheduler.tick(&mut world, Duration::from_secs_f32(1.0 / 60.0));
    assert!(result.is_ok());
}

#[test]
fn test_stage_scheduler_system_ordering_before() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    scheduler.add_system("second", Box::new(|_world| Ok(())));
    scheduler.add_system("first", Box::new(|_world| Ok(()))).before("second");

    let result = scheduler.tick(&mut world, Duration::from_secs_f32(1.0 / 60.0));
    assert!(result.is_ok());
}

#[test]
fn test_stage_scheduler_default() {
    let scheduler = StageScheduler::default();
    assert!(!scheduler.is_startup_executed());
}

#[test]
fn test_fixed_update_default_timestep() {
    let scheduler = StageScheduler::new();
    let expected = Duration::from_secs_f32(1.0 / 60.0);
    let diff = if scheduler.fixed_timestep() > expected {
        scheduler.fixed_timestep() - expected
    }
    else {
        expected - scheduler.fixed_timestep()
    };
    assert!(diff < Duration::from_micros(1));
}

#[test]
fn test_fixed_update_set_timestep() {
    let mut scheduler = StageScheduler::new();
    let new_timestep = Duration::from_secs_f32(1.0 / 30.0);
    scheduler.set_fixed_timestep(new_timestep);
    let diff = if scheduler.fixed_timestep() > new_timestep {
        scheduler.fixed_timestep() - new_timestep
    }
    else {
        new_timestep - scheduler.fixed_timestep()
    };
    assert!(diff < Duration::from_micros(1));
}

#[test]
fn test_fixed_update_normal_frame() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    scheduler.add_system_to_stage(
        "fixed_counter",
        Box::new(move |_world| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
        Stage::FixedUpdate,
    );

    let delta = Duration::from_secs_f32(1.0 / 60.0);
    scheduler.tick(&mut world, delta).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_fixed_update_compensation() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    scheduler.add_system_to_stage(
        "fixed_counter",
        Box::new(move |_world| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
        Stage::FixedUpdate,
    );

    let fixed_timestep = scheduler.fixed_timestep();
    let delta = fixed_timestep * 2;
    scheduler.tick(&mut world, delta).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn test_fixed_update_spiral_of_death_protection() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    scheduler.add_system_to_stage(
        "fixed_counter",
        Box::new(move |_world| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
        Stage::FixedUpdate,
    );

    let delta = Duration::from_secs(10);
    scheduler.tick(&mut world, delta).unwrap();
    assert!(counter.load(Ordering::SeqCst) <= 5);
}

#[test]
fn test_fixed_update_accumulator_residual() {
    let mut scheduler = StageScheduler::new();
    let mut world = World::new();

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    scheduler.add_system_to_stage(
        "fixed_counter",
        Box::new(move |_world| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
        Stage::FixedUpdate,
    );

    let fixed_timestep = scheduler.fixed_timestep();
    let delta = fixed_timestep + fixed_timestep / 2;
    scheduler.tick(&mut world, delta).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    let small_delta = fixed_timestep / 2;
    scheduler.tick(&mut world, small_delta).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}
