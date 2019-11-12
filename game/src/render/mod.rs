mod area;
mod bike;
mod building;
mod bus_stop;
mod car;
mod extra_shape;
mod intersection;
mod lane;
mod map;
mod pedestrian;
mod road;
mod traffic_signal;
mod turn;

use crate::helpers::{ColorScheme, ID};
pub use crate::render::area::DrawArea;
use crate::render::bike::DrawBike;
use crate::render::car::DrawCar;
pub use crate::render::extra_shape::ExtraShapeID;
pub use crate::render::intersection::{calculate_corners, DrawIntersection};
pub use crate::render::lane::DrawLane;
pub use crate::render::map::{AgentCache, AgentColorScheme, DrawMap};
pub use crate::render::pedestrian::{DrawPedCrowd, DrawPedestrian};
pub use crate::render::road::DrawRoad;
pub use crate::render::traffic_signal::{draw_signal_phase, TrafficSignalDiagram};
pub use crate::render::turn::DrawTurn;
use ezgui::{Canvas, Color, GfxCtx, Prerender};
use geom::{Distance, PolyLine, Polygon, Pt2D, EPSILON_DIST};
use map_model::{IntersectionID, Map};
use sim::{DrawCarInput, Sim, VehicleType};
use std::collections::HashMap;

pub const MIN_ZOOM_FOR_DETAIL: f64 = 1.0;

const EXTRA_SHAPE_THICKNESS: Distance = Distance::const_meters(1.0);
const EXTRA_SHAPE_POINT_RADIUS: Distance = Distance::const_meters(10.0);

const BIG_ARROW_THICKNESS: Distance = Distance::const_meters(0.5);

const TURN_ICON_ARROW_THICKNESS: Distance = Distance::const_meters(0.15);
const TURN_ICON_ARROW_LENGTH: Distance = Distance::const_meters(2.0);
pub const CROSSWALK_LINE_THICKNESS: Distance = Distance::const_meters(0.25);

pub const OUTLINE_THICKNESS: Distance = Distance::const_meters(0.5);

// Does something belong here or as a method on ID? If it ONLY applies to renderable things, then
// here. For example, trips aren't drawn, so it's meaningless to ask what their bounding box is.
pub trait Renderable {
    // TODO This is expensive for the PedCrowd case. :( Returning a borrow is awkward, because most
    // Renderables are better off storing the inner ID directly.
    fn get_id(&self) -> ID;
    fn draw(&self, g: &mut GfxCtx, opts: &DrawOptions, ctx: &DrawCtx);
    // Higher z-ordered objects are drawn later. Default to low so roads at -1 don't vanish.
    fn get_zorder(&self) -> isize {
        -5
    }
    // This outline is drawn over the base object to show that it's selected. It also represents
    // the boundaries for quadtrees. This isn't called often; don't worry about caching.
    fn get_outline(&self, map: &Map) -> Polygon;
    fn contains_pt(&self, pt: Pt2D, map: &Map) -> bool {
        self.get_outline(map).contains_pt(pt)
    }
}

pub fn draw_vehicle(
    input: DrawCarInput,
    map: &Map,
    prerender: &Prerender,
    canvas: &Canvas,
    cs: &ColorScheme,
    acs: AgentColorScheme,
    use_textures: bool,
) -> Box<dyn Renderable> {
    if input.id.1 == VehicleType::Bike {
        Box::new(DrawBike::new(
            input,
            map,
            prerender,
            canvas,
            cs,
            acs,
            use_textures,
        ))
    } else {
        Box::new(DrawCar::new(
            input,
            map,
            prerender,
            canvas,
            cs,
            acs,
            use_textures,
        ))
    }
}

pub fn dashed_lines(
    pl: &PolyLine,
    width: Distance,
    dash_len: Distance,
    dash_separation: Distance,
) -> Vec<Polygon> {
    if pl.length() < dash_separation * 2.0 + EPSILON_DIST {
        return vec![pl.make_polygons(width)];
    }
    // Don't draw the dashes too close to the ends.
    pl.exact_slice(dash_separation, pl.length() - dash_separation)
        .dashed_polygons(width, dash_len, dash_separation)
}

pub struct DrawCtx<'a> {
    pub cs: &'a ColorScheme,
    pub map: &'a Map,
    pub draw_map: &'a DrawMap,
    pub sim: &'a Sim,
}

pub struct DrawOptions {
    pub override_colors: HashMap<ID, Color>,
    pub suppress_traffic_signal_details: Option<IntersectionID>,
    pub label_buildings: bool,
    pub label_roads: bool,
}

impl DrawOptions {
    pub fn new() -> DrawOptions {
        DrawOptions {
            override_colors: HashMap::new(),
            suppress_traffic_signal_details: None,
            label_buildings: false,
            label_roads: false,
        }
    }

    pub fn color(&self, id: ID) -> Option<Color> {
        self.override_colors.get(&id).cloned()
    }
}

pub fn osm_rank_to_unzoomed_color(cs: &ColorScheme, rank: usize) -> Color {
    if rank >= 16 {
        cs.get_def("unzoomed highway road", Color::rgb(232, 146, 162))
    } else if rank >= 6 {
        cs.get_def("unzoomed arterial road", Color::rgb(255, 199, 62))
    } else {
        cs.get_def("unzoomed residential road", Color::WHITE)
    }
}

pub fn osm_rank_to_zoomed_color(cs: &ColorScheme, rank: usize) -> Color {
    if rank >= 16 {
        cs.get_def("zoomed residential road", Color::from_hex("#7c7c7c"))
    } else if rank >= 6 {
        cs.get_def("zoomed highway road", Color::from_hex("#2a2a2a"))
    } else {
        cs.get_def("zoomed arterial road", Color::from_hex("#5b5b5b"))
    }
}

pub fn osm_rank_to_road_center_line_color(cs: &ColorScheme, rank: usize) -> Color {
    if rank >= 16 {
        cs.get_def("residential center line", Color::from_hex("#f4da22"))
    } else if rank >= 6 {
        cs.get_def("highway center line", Color::from_hex("#db952e"))
    } else {
        cs.get_def("arterial center line", Color::from_hex("#d8b830"))
    }
}
