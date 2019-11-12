use crate::helpers::{ColorScheme, ID};
use crate::render::{
    dashed_lines, osm_rank_to_road_center_line_color, osm_rank_to_zoomed_color, DrawCtx,
    DrawOptions, Renderable, OUTLINE_THICKNESS,
};
use abstutil::Timer;
use ezgui::{Color, Drawable, GeomBatch, GfxCtx, Prerender};
use geom::{Distance, Line, PolyLine, Polygon, Pt2D};
use map_model::{Lane, LaneID, LaneType, Map, Road, TurnType, LANE_THICKNESS, PARKING_SPOT_LENGTH};

// Split into two phases like this, because AlmostDrawLane can be created in parallel, but GPU
// upload has to be serial.
pub struct AlmostDrawLane {
    id: LaneID,
    polygon: Polygon,
    zorder: isize,
    draw_default: GeomBatch,
}

impl AlmostDrawLane {
    pub fn finish(self, prerender: &Prerender) -> DrawLane {
        DrawLane {
            id: self.id,
            polygon: self.polygon,
            zorder: self.zorder,
            draw_default: prerender.upload(self.draw_default),
        }
    }
}

pub struct DrawLane {
    pub id: LaneID,
    pub polygon: Polygon,
    zorder: isize,

    draw_default: Drawable,
}

impl DrawLane {
    pub fn new(
        lane: &Lane,
        map: &Map,
        draw_lane_markings: bool,
        cs: &ColorScheme,
        timer: &mut Timer,
    ) -> AlmostDrawLane {
        let road = map.get_r(lane.parent);
        let rank = road.get_rank();
        let polygon = lane.lane_center_pts.make_polygons(LANE_THICKNESS);

        let mut draw = GeomBatch::new();
        draw.push(
            match lane.lane_type {
                LaneType::Driving | LaneType::Parking | LaneType::SharedLeftTurn => {
                    osm_rank_to_zoomed_color(cs, rank)
                }
                LaneType::Bus => cs.get_def("bus lane", Color::from_hex("#ad302d")),
                LaneType::Sidewalk => cs.get_def("sidewalk", Color::from_hex("#d6d6d6")),
                LaneType::Biking => cs.get_def("bike lane", Color::from_hex("#72ce36")),
                LaneType::Construction => {
                    cs.get_def("construction background", Color::rgb(255, 109, 0))
                }
            },
            polygon.clone(),
        );
        if draw_lane_markings {
            match lane.lane_type {
                LaneType::Sidewalk => {
                    draw.extend(
                        cs.get_def("sidewalk lines", Color::from_hex("#707070")),
                        calculate_sidewalk_lines(lane),
                    );
                }
                LaneType::Parking => {
                    draw.extend(
                        cs.get_def("general road marking", Color::from_hex("#d6d6d6")),
                        calculate_parking_lines(lane),
                    );
                }
                LaneType::Driving | LaneType::Bus => {
                    draw.extend(
                        cs.get("general road marking"),
                        calculate_driving_lines(lane, road, timer),
                    );
                    draw.extend(
                        cs.get("general road marking"),
                        calculate_turn_markings(map, lane, timer),
                    );
                }
                LaneType::Biking => {}
                LaneType::SharedLeftTurn => {
                    let color = osm_rank_to_road_center_line_color(cs, rank);
                    draw.push(
                        color,
                        lane.lane_center_pts
                            .shift_right(LANE_THICKNESS / 2.0)
                            .get(timer)
                            .make_polygons(Distance::meters(0.25)),
                    );
                    draw.push(
                        color,
                        lane.lane_center_pts
                            .shift_left(LANE_THICKNESS / 2.0)
                            .get(timer)
                            .make_polygons(Distance::meters(0.25)),
                    );
                }
                LaneType::Construction => {
                    draw.push(
                        cs.get_def("construction hatching", Color::HatchingStyle2),
                        polygon.clone(),
                    );
                }
            };
        }

        AlmostDrawLane {
            id: lane.id,
            polygon,
            zorder: road.get_zorder(),
            draw_default: draw,
        }
    }
}

impl Renderable for DrawLane {
    fn get_id(&self) -> ID {
        ID::Lane(self.id)
    }

    fn draw(&self, g: &mut GfxCtx, opts: &DrawOptions, _: &DrawCtx) {
        if let Some(color) = opts.color(self.get_id()) {
            g.draw_polygon(color, &self.polygon);
        } else {
            g.redraw(&self.draw_default);
        }
    }

    fn get_outline(&self, map: &Map) -> Polygon {
        map.get_l(self.id)
            .lane_center_pts
            .to_thick_boundary(LANE_THICKNESS, OUTLINE_THICKNESS)
            .unwrap_or_else(|| self.polygon.clone())
    }

    fn contains_pt(&self, pt: Pt2D, _: &Map) -> bool {
        self.polygon.contains_pt(pt)
    }

    fn get_zorder(&self) -> isize {
        self.zorder
    }
}

// TODO this always does it at pt1
fn perp_line(l: Line, length: Distance) -> Line {
    let pt1 = l.shift_right(length / 2.0).pt1();
    let pt2 = l.shift_left(length / 2.0).pt1();
    Line::new(pt1, pt2)
}

fn calculate_sidewalk_lines(lane: &Lane) -> Vec<Polygon> {
    let tile_every = LANE_THICKNESS;

    let length = lane.length();

    let mut result = Vec::new();
    // Start away from the intersections
    let mut dist_along = tile_every;
    while dist_along < length - tile_every {
        let (pt, angle) = lane.dist_along(dist_along);
        // Reuse perp_line. Project away an arbitrary amount
        let pt2 = pt.project_away(Distance::meters(1.0), angle);
        result.push(
            perp_line(Line::new(pt, pt2), LANE_THICKNESS).make_polygons(Distance::meters(0.25)),
        );
        dist_along += tile_every;
    }

    result
}

fn calculate_parking_lines(lane: &Lane) -> Vec<Polygon> {
    let leg_length = Distance::meters(1.0);

    let mut result = Vec::new();
    let num_spots = lane.number_parking_spots();
    if num_spots > 0 {
        for idx in 0..=num_spots {
            let (pt, lane_angle) = lane.dist_along(PARKING_SPOT_LENGTH * (1.0 + idx as f64));
            let perp_angle = lane_angle.rotate_degs(270.0);
            // Find the outside of the lane. Actually, shift inside a little bit, since the line will
            // have thickness, but shouldn't really intersect the adjacent line when drawn.
            let t_pt = pt.project_away(LANE_THICKNESS * 0.3, perp_angle);
            // The perp leg
            let p1 = t_pt.project_away(leg_length, perp_angle.opposite());
            result.push(Line::new(t_pt, p1).make_polygons(Distance::meters(0.25)));
            // Upper leg
            let p2 = t_pt.project_away(leg_length, lane_angle);
            result.push(Line::new(t_pt, p2).make_polygons(Distance::meters(0.25)));
            // Lower leg
            let p3 = t_pt.project_away(leg_length, lane_angle.opposite());
            result.push(Line::new(t_pt, p3).make_polygons(Distance::meters(0.25)));
        }
    }

    result.push(
        lane.lane_center_pts
            .shift_left(LANE_THICKNESS / 2.0)
            .unwrap()
            .make_polygons(Distance::meters(0.25)),
    );

    result
}

fn calculate_driving_lines(lane: &Lane, parent: &Road, timer: &mut Timer) -> Vec<Polygon> {
    // The leftmost lanes don't have dashed lines.
    let (dir, idx) = parent.dir_and_offset(lane.id);
    if idx == 0 || (dir && parent.children_forwards[idx - 1].1 == LaneType::SharedLeftTurn) {
        return Vec::new();
    }
    let lane_edge_pts = lane
        .lane_center_pts
        .shift_left(LANE_THICKNESS / 2.0)
        .get(timer);
    dashed_lines(
        &lane_edge_pts,
        Distance::meters(0.25),
        Distance::meters(1.0),
        Distance::meters(1.5),
    )
}

fn calculate_turn_markings(map: &Map, lane: &Lane, timer: &mut Timer) -> Vec<Polygon> {
    let mut results = Vec::new();

    // Are there multiple driving lanes on this side of the road?
    if map
        .find_closest_lane(lane.id, vec![LaneType::Driving])
        .is_err()
    {
        return results;
    }
    if lane.length() < Distance::meters(7.0) {
        return results;
    }

    let thickness = Distance::meters(0.2);

    let common_base = lane.lane_center_pts.exact_slice(
        lane.length() - Distance::meters(7.0),
        lane.length() - Distance::meters(5.0),
    );
    results.push(common_base.make_polygons(thickness));

    // TODO Maybe draw arrows per target road, not lane
    for turn in map.get_turns_from_lane(lane.id) {
        if turn.turn_type == TurnType::LaneChangeLeft || turn.turn_type == TurnType::LaneChangeRight
        {
            continue;
        }
        results.push(
            PolyLine::new(vec![
                common_base.last_pt(),
                common_base
                    .last_pt()
                    .project_away(LANE_THICKNESS / 2.0, turn.angle()),
            ])
            .make_arrow(thickness)
            .with_context(timer, format!("turn_markings for {}", turn.id)),
        );
    }

    // Just lane-changing turns after all (common base + 2 for the arrow)
    if results.len() == 3 {
        return Vec::new();
    }
    results
}
