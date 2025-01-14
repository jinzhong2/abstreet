use crate::helpers::ID;
use crate::render::{DrawCtx, DrawOptions, Renderable};
use ezgui::{Color, EventCtx, GeomBatch, GfxCtx};
use geom::Polygon;
use map_model::{Area, AreaID, AreaType, Map};

pub struct DrawArea {
    pub id: AreaID,
}

impl DrawArea {
    pub fn new(area: &Area, ctx: &EventCtx, all_areas: &mut GeomBatch) -> DrawArea {
        let color = match area.area_type {
            AreaType::Park => ctx.canvas.texture("assets/grass_texture.png"),
            AreaType::Water => ctx.canvas.texture("assets/water_texture.png"),
            AreaType::PedestrianIsland => Color::grey(0.3),
        };
        all_areas.push(color, area.polygon.clone());
        DrawArea { id: area.id }
    }
}

impl Renderable for DrawArea {
    fn get_id(&self) -> ID {
        ID::Area(self.id)
    }

    fn draw(&self, g: &mut GfxCtx, opts: &DrawOptions, ctx: &DrawCtx) {
        if let Some(color) = opts.color(self.get_id()) {
            g.draw_polygon(color, &ctx.map.get_a(self.id).polygon);
        }
    }

    fn get_outline(&self, map: &Map) -> Polygon {
        // Since areas are so big, don't just draw the outline
        map.get_a(self.id).polygon.clone()
    }
}
