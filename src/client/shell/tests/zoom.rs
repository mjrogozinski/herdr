use super::*;

fn zoomed_state(chrome: u16) -> ClientShellState {
    let mut config = Config::default();
    config.ui.zoom_width = std::num::NonZeroU16::new(20);
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&config));
    let mut projection = snapshot();
    projection.tabs[0].zoomed = true;
    state.set_snapshot(Box::new(projection));
    let mut frame = surface();
    frame.panes[0].rect.width = 40;
    frame.panes[0].inner_rect.width = 40 - chrome;
    state.set_pane_surface(frame);
    state
}

#[test]
fn configured_zoom_width_centers_content_and_clamps_to_available_area() {
    for chrome in [0, 1, 2, 3] {
        let state = zoomed_state(chrome);
        let full = state.config.layout(
            120,
            30,
            state.sidebar_collapsed,
            state.focused_tab_count(),
            state.sidebar_width,
        );
        let centered = state.layout(120, 30);
        assert_eq!(centered.pane_surface.width, 20 + chrome);
        assert_eq!(
            centered.pane_surface.x,
            full.pane_surface.x + (full.pane_surface.width - 20 - chrome) / 2
        );
        assert_eq!(centered.sidebar, full.sidebar);
        assert_eq!(centered.tab_bar, full.tab_bar);
        assert_eq!(state.layout(10, 30).pane_surface.width, 10);
    }
}

#[test]
fn zoom_width_tracks_zoom_toggle_and_config_reload() {
    let mut state = zoomed_state(3);
    assert_eq!(state.surface_size(120, 30).cols, 23);
    let mut projection = snapshot();
    projection.revision = 2;
    state.set_snapshot(Box::new(projection.clone()));
    assert!(state.surface_size(120, 30).cols > 23);
    projection.revision = 3;
    projection.tabs[0].zoomed = true;
    state.set_snapshot(Box::new(projection));
    assert_eq!(state.surface_size(120, 30).cols, 23);
    let mut config = Config::default();
    config.ui.zoom_width = std::num::NonZeroU16::new(30);
    assert!(state.config.apply_live_config(&config, &[], &[]).is_empty());
    assert_eq!(state.surface_size(120, 30).cols, 33);
}

#[test]
fn zoom_width_resize_uses_client_geometry_and_preserves_chrome() {
    let mut state = zoomed_state(3);
    state.mode = ClientShellMode::Resize;
    for (action, expected) in [
        (crate::input::KeybindAction::ResizePaneRight, 21),
        (crate::input::KeybindAction::ResizePaneLeft, 20),
    ] {
        let mut outcome = ClientShellInput::default();
        state.record_binding(crate::input::KeybindMatch::Action(action), &mut outcome);
        assert!(outcome.resize);
        assert!(outcome.repaint);
        assert_eq!(
            state.config.zoom_width.map(std::num::NonZeroU16::get),
            Some(expected)
        );
        assert_eq!(state.surface_size(120, 30).cols, expected + 3);
        assert_eq!(state.mode, ClientShellMode::Resize);
    }
}

#[test]
fn zoom_width_follows_changed_terminal_chrome() {
    let mut state = zoomed_state(3);
    assert_eq!(state.surface_size(120, 30).cols, 23);
    let mut frame = surface();
    frame.surface_revision = 2;
    frame.panes[0].rect.width = 40;
    frame.panes[0].inner_rect.width = 38;
    frame.panes[0].alternate_screen_active = true;
    state.set_pane_surface(frame);
    assert_eq!(state.surface_size(120, 30).cols, 22);
}

#[test]
fn zoom_width_keeps_mouse_hit_coordinates_in_the_centered_surface() {
    let mut state = zoomed_state(3);
    let area = state.layout(120, 30).pane_surface;
    let mut frame = surface();
    frame.surface_revision = 2;
    frame.frame = FrameData::from_ratatui_buffer_with_hyperlinks(
        &Buffer::empty(Rect::new(0, 0, area.width, area.height)),
        None,
        &[],
    );
    frame.panes[0].rect = SurfaceRect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    frame.panes[0].inner_rect = SurfaceRect {
        x: 1,
        y: 1,
        width: area.width - 3,
        height: area.height - 2,
    };
    state.set_pane_surface(frame);
    assert!(state.compose(120, 30).is_some());
    assert_eq!(state.hits.panes[0].rect.x, area.x);
    assert_eq!(state.hits.panes[0].inner_rect.x, area.x + 1);
    assert_eq!(state.hits.panes[0].inner_rect.width, 20);
}
