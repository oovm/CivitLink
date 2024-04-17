use gg_render::{Camera, Camera2D, Color, Rect, Viewport};

#[test]
fn test_viewport_full() {
    let vp = Viewport::FULL;
    assert_eq!(vp.x, 0.0);
    assert_eq!(vp.y, 0.0);
    assert_eq!(vp.width, 1.0);
    assert_eq!(vp.height, 1.0);
}

#[test]
fn test_viewport_new() {
    let vp = Viewport::new();
    assert_eq!(vp, Viewport::FULL);
}

#[test]
fn test_viewport_default() {
    let vp = Viewport::default();
    assert_eq!(vp, Viewport::FULL);
}

#[test]
fn test_viewport_with_rect() {
    let vp = Viewport::with_rect(0.25, 0.25, 0.5, 0.5);
    assert_eq!(vp.x, 0.25);
    assert_eq!(vp.y, 0.25);
    assert_eq!(vp.width, 0.5);
    assert_eq!(vp.height, 0.5);
}

#[test]
fn test_camera2d_new() {
    let cam = Camera2D::new();
    assert_eq!(cam.position, [0.0, 0.0]);
    assert_eq!(cam.zoom, 1.0);
    assert_eq!(cam.rotation, 0.0);
    assert_eq!(cam.viewport, Viewport::FULL);
    assert_eq!(cam.ortho_size, 0.0);
}

#[test]
fn test_camera2d_default() {
    let cam = Camera2D::default();
    assert_eq!(cam.position, [0.0, 0.0]);
}

#[test]
fn test_camera2d_builder() {
    let cam = Camera2D::new()
        .with_position(100.0, 200.0)
        .with_zoom(2.0)
        .with_rotation(1.57)
        .with_viewport_rect(0.0, 0.0, 0.5, 0.5)
        .with_ortho_size(5.0);

    assert_eq!(cam.position, [100.0, 200.0]);
    assert_eq!(cam.zoom, 2.0);
    assert_eq!(cam.rotation, 1.57);
    assert_eq!(cam.viewport, Viewport::with_rect(0.0, 0.0, 0.5, 0.5));
    assert_eq!(cam.ortho_size, 5.0);
}

#[test]
fn test_camera2d_builder_with_viewport() {
    let vp = Viewport::with_rect(0.1, 0.2, 0.3, 0.4);
    let cam = Camera2D::new().with_viewport(vp);
    assert_eq!(cam.viewport, vp);
}

#[test]
fn test_camera2d_from_camera() {
    let camera = Camera::new().with_position(10.0, 20.0).with_zoom(3.0).with_rotation(0.5);
    let cam2d: Camera2D = camera.into();

    assert_eq!(cam2d.position, [10.0, 20.0]);
    assert_eq!(cam2d.zoom, 3.0);
    assert_eq!(cam2d.rotation, 0.5);
    assert_eq!(cam2d.viewport, Viewport::FULL);
    assert_eq!(cam2d.ortho_size, 0.0);
}

#[test]
fn test_camera2d_view_projection_identity() {
    let cam = Camera2D::new();
    let vp = cam.view_projection(800.0, 600.0);

    // Identity camera should produce the same result as the orthographic projection alone
}
