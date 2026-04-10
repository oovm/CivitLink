use gg_schedule::prelude::*;
use gg_ecs::World;

fn result_system(_world: &mut World) -> gg_error::GResult<()> {
    Ok(())
}

fn void_system(_world: &mut World) {}

#[test]
fn test_into_system_result_fn() {
    let mut wrapper: SystemWrapper = result_system.into_system();
    assert!(!wrapper.name.is_empty());
    let mut world = World::new();
    (wrapper.func)(&mut world).unwrap();
}

#[test]
fn test_into_system_void_fn() {
    let mut wrapper: SystemWrapper = void_system.into_system();
    assert!(!wrapper.name.is_empty());
    let mut world = World::new();
    (wrapper.func)(&mut world).unwrap();
}

#[test]
fn test_schedules_run_startup_once() {
    let mut schedules = Schedules::new();
    let mut world = World::new();

    schedules.add_startup_systems(result_system);
    schedules.run_startup(&mut world).unwrap();
    schedules.run_startup(&mut world).unwrap();
}

#[test]
fn test_schedules_run_update_every_time() {
    let mut schedules = Schedules::new();
    let mut world = World::new();

    schedules.add_update_systems(void_system);
    schedules.run_update(&mut world).unwrap();
    schedules.run_update(&mut world).unwrap();
}

#[test]
fn test_topological_sort_with_dependencies() {
    let mut schedule = Schedule::new(Update);

    schedule.add_systems(SystemWrapper::new("c", |_: &mut World| Ok(())).after_system("b"));
    schedule.add_systems(SystemWrapper::new("a", |_: &mut World| Ok(())).before_system("b"));
    schedule.add_systems(SystemWrapper::new("b", |_: &mut World| Ok(())));

    let order = schedule.topological_sort().unwrap();
    let names: Vec<&str> = order.iter().map(|&i| schedule.systems[i].name.as_str()).collect();

    let pos_a = names.iter().position(|&n| n == "a").unwrap();
    let pos_b = names.iter().position(|&n| n == "b").unwrap();
    let pos_c = names.iter().position(|&n| n == "c").unwrap();

    assert!(pos_a < pos_b, "a should run before b");
    assert!(pos_b < pos_c, "b should run before c");
}

#[test]
fn test_schedules_getters() {
    let schedules = Schedules::new();

    assert_eq!(schedules.startup_schedule().label.label_name(), "Startup");
    assert_eq!(schedules.update_schedule().label.label_name(), "Update");
    assert_eq!(schedules.fixed_update_schedule().label.label_name(), "FixedUpdate");
    assert_eq!(schedules.post_update_schedule().label.label_name(), "PostUpdate");
    assert_eq!(schedules.render_schedule().label.label_name(), "Render");
    assert_eq!(schedules.exit_schedule().label.label_name(), "Exit");
}

#[test]
fn test_schedule_add_systems_with_fn() {
    let mut schedule = Schedule::new(Update);
    schedule.add_systems(result_system);
    schedule.add_systems(void_system);

    assert_eq!(schedule.systems.len(), 2);

    let mut world = World::new();
    schedule.run(&mut world).unwrap();
}
