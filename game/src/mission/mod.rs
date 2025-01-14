mod all_trips;
mod dataviz;
mod individ_trips;
mod neighborhood;
mod scenario;

use crate::game::{State, Transition, WizardState};
use crate::ui::UI;
use abstutil::Timer;
use ezgui::{hotkey, EventCtx, GfxCtx, Key, ModalMenu, Wizard, WrappedWizard};
use geom::Duration;
use sim::Scenario;

pub struct MissionEditMode {
    menu: ModalMenu,
}

impl MissionEditMode {
    pub fn new(ctx: &EventCtx) -> MissionEditMode {
        MissionEditMode {
            menu: ModalMenu::new(
                "Mission Edit Mode",
                vec![
                    (hotkey(Key::D), "visualize population data"),
                    (hotkey(Key::T), "visualize individual PSRC trips"),
                    (hotkey(Key::A), "visualize all PSRC trips"),
                    (hotkey(Key::N), "manage neighborhoods"),
                    (hotkey(Key::W), "load scenario"),
                    (None, "create new scenario"),
                    (hotkey(Key::Escape), "quit"),
                ],
                ctx,
            ),
        }
    }
}

impl State for MissionEditMode {
    fn event(&mut self, ctx: &mut EventCtx, ui: &mut UI) -> Transition {
        self.menu.event(ctx);
        ctx.canvas.handle_event(ctx.input);

        if self.menu.action("quit") {
            return Transition::Pop;
        } else if self.menu.action("visualize population data") {
            return Transition::Push(Box::new(dataviz::DataVisualizer::new(ctx, ui)));
        } else if self.menu.action("visualize individual PSRC trips") {
            return Transition::Push(Box::new(individ_trips::TripsVisualizer::new(ctx, ui)));
        } else if self.menu.action("visualize all PSRC trips") {
            return Transition::Push(Box::new(all_trips::TripsVisualizer::new(ctx, ui)));
        } else if self.menu.action("manage neighborhoods") {
            return Transition::Push(Box::new(neighborhood::NeighborhoodPicker::new()));
        } else if self.menu.action("load scenario") {
            return Transition::Push(WizardState::new(Box::new(load_scenario)));
        } else if self.menu.action("create new scenario") {
            return Transition::Push(WizardState::new(Box::new(create_new_scenario)));
        }
        Transition::Keep
    }

    fn draw(&self, g: &mut GfxCtx, _: &UI) {
        self.menu.draw(g);
    }
}

fn load_scenario(wiz: &mut Wizard, ctx: &mut EventCtx, ui: &mut UI) -> Option<Transition> {
    let map_name = ui.primary.map.get_name().to_string();
    let s = wiz.wrap(ctx).choose_string("Load which scenario?", || {
        abstutil::list_all_objects(abstutil::SCENARIOS, &map_name)
    })?;
    let scenario = abstutil::read_binary(
        &abstutil::path1_bin(&map_name, abstutil::SCENARIOS, &s),
        &mut Timer::throwaway(),
    )
    .unwrap();
    Some(Transition::Replace(Box::new(
        scenario::ScenarioManager::new(scenario, ctx, ui),
    )))
}

fn create_new_scenario(wiz: &mut Wizard, ctx: &mut EventCtx, ui: &mut UI) -> Option<Transition> {
    let name = wiz.wrap(ctx).input_string("Name the scenario")?;
    let mut s = Scenario::empty(&ui.primary.map);
    s.seed_buses = true;
    s.scenario_name = name;
    Some(Transition::Replace(Box::new(
        scenario::ScenarioManager::new(s, ctx, ui),
    )))
}

pub fn pick_time_range(
    wizard: &mut WrappedWizard,
    low_query: &str,
    high_query: &str,
) -> Option<(Duration, Duration)> {
    let t1 = wizard.input_time_slider(low_query, Duration::ZERO, Duration::END_OF_DAY)?;
    let t2 = wizard.input_time_slider(high_query, t1, Duration::END_OF_DAY)?;
    Some((t1, t2))
}
