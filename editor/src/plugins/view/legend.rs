use crate::objects::Ctx;
use crate::plugins::{Plugin, PluginCtx};
use crate::render::DrawTurn;
use ezgui::{GfxCtx, ScreenPt, Text};
use geom::{PolyLine, Pt2D};
use map_model::{IntersectionID, LaneID, Turn, TurnID, TurnType};

pub struct Legend {
    top_left: ScreenPt,
}

impl Legend {
    pub fn new(ctx: &mut PluginCtx) -> Option<Legend> {
        if ctx.input.action_chosen("show legend") {
            return Some(Legend {
                top_left: ctx.input.set_mode("Legend", &ctx.canvas),
            });
        }
        None
    }
}

impl Plugin for Legend {
    fn nonblocking_event(&mut self, ctx: &mut PluginCtx) -> bool {
        self.top_left = ctx.input.set_mode("Legend", &ctx.canvas);

        // TODO Hack
        self.top_left.x -= 150.0;
        if ctx.input.modal_action("quit") {
            return false;
        }
        true
    }

    fn draw(&self, g: &mut GfxCtx, ctx: &Ctx) {
        // TODO The negation and reasoning about the zoom is annoying. I want to say something like
        // "Make top_left the origin, zoom 10."
        let zoom = 10.0;
        let old_ctx = g.fork(
            Pt2D::new(-self.top_left.x / zoom, -self.top_left.y / zoom),
            zoom,
        );

        // Create a fake turn.
        let mut turn = Turn {
            id: TurnID {
                parent: IntersectionID(0),
                src: LaneID(0),
                dst: LaneID(0),
            },
            turn_type: TurnType::Straight,
            lookup_idx: 0,
            geom: PolyLine::new(vec![
                Pt2D::new(10.0 / zoom, 10.0 / zoom),
                Pt2D::new(10.0 / zoom, 100.0 / zoom),
            ]),
        };

        DrawTurn::draw_full(
            &turn,
            g,
            ctx.cs.get("turns protected by traffic signal right now"),
        );
        ctx.canvas.draw_text_at_screenspace_topleft(
            g,
            Text::from_line("Protected turn".to_string()),
            ScreenPt::new(self.top_left.x + 20.0, self.top_left.y + 10.0),
        );

        turn.geom = PolyLine::new(vec![
            Pt2D::new(10.0 / zoom, 110.0 / zoom),
            Pt2D::new(10.0 / zoom, 200.0 / zoom),
        ]);
        DrawTurn::draw_dashed(
            &turn,
            g,
            ctx.cs
                .get("turns allowed with yielding by traffic signal right now"),
        );
        ctx.canvas.draw_text_at_screenspace_topleft(
            g,
            Text::from_line("Yield turn".to_string()),
            ScreenPt::new(self.top_left.x + 20.0, self.top_left.y + 110.0),
        );

        g.unfork(old_ctx);
    }
}