use few_pretty_graphs::{dbscan_parents, Label};
use geoprofile_core::{Label as GeoLabel, Row, Corner, Face};
//use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};
use idek::{prelude::*, MultiPlatformCamera};
use std::{path::PathBuf, ops::BitXor};
use structopt::StructOpt;

fn main() -> Result<()> {
    // TODO: get VR as arg. Must modify idek...
    let opt = Opt::from_args();
    launch::<Opt, FewPrettyGraphs>(Settings::default().vr(opt.vr).args(opt))
}

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "A few pretty graphs", about = "DBSCAN go brrrrr")]
struct Opt {
    /// Model
    obj_path: PathBuf,

    /// Cluster radius
    #[structopt(short, long)]
    radius: f32,

    /// Cluster minimum points
    #[structopt(short, long)]
    min_pts: usize,

    /// Use VR
    #[structopt(short, long)]
    vr: bool,
}

struct FewPrettyGraphs {
    verts: VertexBuffer,
    line_indices: IndexBuffer,
    points_shader: Shader,
    lines_shader: Shader,
    camera: MultiPlatformCamera,
}

fn u64_color(u: u64) -> [f32; 3] {
    let mut rgb = [0u8; 3];
    rgb.copy_from_slice(&u.to_le_bytes()[..3]);
    rgb.map(|x| x as f32 / u8::MAX as f32)
}

impl App<Opt> for FewPrettyGraphs {
    fn init(ctx: &mut Context, platform: &mut Platform, args: Opt) -> Result<Self> {
        let board = geoprofile_core::load_board(args.obj_path)?;
        let sets = extract_label_sets(&board);
        let label_idx: i32 = GeoLabel::Wood(Face::Top).into();
        let points = &sets[label_idx as usize].1;

        let time = std::time::Instant::now();
        let (n_clusters, labels) = dbscan_parents(points, args.radius, args.min_pts);
        dbg!(time.elapsed().as_secs_f32());

        dbg!(n_clusters);

        let color_lut: Vec<[f32; 3]> = trivial_random(389204)
            .map(u64_color)
            .take(n_clusters as _)
            .collect();

        let line_indices: Vec<u32> = labels
            .iter()
            .enumerate()
            .filter_map(|(idx, label)| match label {
                Label::Cluster { prev, .. } => Some([idx as u32, *prev as u32]),
                _ => None,
            })
            .flatten()
            .collect();

        dbg!(line_indices.len());

        let points_vertices: Vec<Vertex> = points
            .into_iter()
            .zip(labels)
            .map(|(&pos, label)| Vertex {
                pos,
                color: match label {
                    Label::Undefined => [1., 0., 1.],
                    Label::Noise => [0.5; 3],
                    Label::Cluster { id, .. } => color_lut[id as usize],
                },
            })
            .collect();

        Ok(Self {
            points_shader: ctx.shader(
                include_bytes!("shaders/points.vert.spv"),
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Points,
            )?,
            lines_shader: ctx.shader(
                DEFAULT_VERTEX_SHADER,
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Lines,
            )?,
            verts: ctx.vertices(&points_vertices, false)?,
            line_indices: ctx.indices(&line_indices, false)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![
            DrawCmd::new(self.verts).shader(self.points_shader),
            DrawCmd::new(self.verts)
                .shader(self.lines_shader)
                .indices(self.line_indices),
        ])
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

fn extract_label_sets(rows: &[Row]) -> Vec<(GeoLabel, Vec<[f32; 3]>)> {
    const DOWNSCALE: f32 = 1000.;
    let mut sets: Vec<(GeoLabel, Vec<[f32; 3]>)> = (0..GeoLabel::N_CLASSES)
        .map(|idx| (GeoLabel::from(idx as i32), vec![]))
        .collect();

    for row in rows {
        if let Some(labels) = row.labels.as_ref() {
            for (point, &label) in row.points.iter().zip(labels) {
                let pos = [point.x, point.y, row.length_position].map(|v| v / DOWNSCALE);
                let set_idx: i32 = label.into();
                sets[set_idx as usize].1.push(pos);
            }
        }
    }

    sets
}

/// FxHasher
/// https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html
pub struct FxHasher {
    pub hash: u64,
}

impl Default for FxHasher {
    #[inline]
    fn default() -> Self {
        Self {
            hash: 0x4234234234129,
        }
    }
}

impl FxHasher {
    const K: u64 = 0x517cc1b727220a95;

    #[inline]
    pub fn add_to_hash(&mut self, i: u64) {
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(Self::K);
    }

    #[inline]
    pub fn finish(&self) -> u64 {
        self.hash as u64
    }
}

/// Produces a stream of pseudorandom values
fn trivial_random(seed: u64) -> impl Iterator<Item = u64> {
    let mut hasher = FxHasher { hash: seed };
    (0..).map(move |x| {
        hasher.add_to_hash(x);
        hasher.finish()
    })
}
