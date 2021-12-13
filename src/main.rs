use few_pretty_graphs::obj::load_obj_verts;
//use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};
use idek::{prelude::*, MultiPlatformCamera};

fn main() -> Result<()> {
    launch::<FewPrettyGraphs>(Settings::default().vr_if_any_args())
}

struct FewPrettyGraphs {
    verts: VertexBuffer,
    //indices: IndexBuffer,
    shader: Shader,
    camera: MultiPlatformCamera,
}

impl App for FewPrettyGraphs {
    fn init(ctx: &mut Context, platform: &mut Platform) -> Result<Self> {
        let vertices = load_obj_verts("models/bigbunny.obj")?;
        let vertices: Vec<Vertex> = vertices.into_iter().map(|pos| Vertex {
            pos,
            color: [1.; 3],
        }).collect();

        //let indices = (0..vertices.len()).collect();

        Ok(Self {
            shader: ctx.shader(
                include_bytes!("shaders/points.vert.spv"),
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Points,
            )?,
            verts: ctx.vertices(&vertices, false)?,
            //indices: ctx.indices(&indices, false)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)
            //.indices(self.indices)
            .shader(self.shader)])
    }

    fn event(
        &mut self,
        ctx: &mut Context,
        platform: &mut Platform,
        mut event: Event,
    ) -> Result<()> {
        if self.camera.handle_event(&mut event) {
            ctx.set_camera_prefix(self.camera.get_prefix())
        }
        idek::close_when_asked(platform, &event);
        Ok(())
    }
}

/*
fn rainbow_cube() -> (Vec<Vertex>, Vec<u32>) {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    (vertices, indices)
}
*/
