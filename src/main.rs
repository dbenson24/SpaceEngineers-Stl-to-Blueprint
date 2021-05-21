use float_ord::FloatOrd;
use rayon::prelude::*;
use std::cmp;
use std::fmt;
use std::fs::OpenOptions;
extern crate nalgebra as na;

use itertools::Itertools;

use na::{distance, distance_squared, Point3, Unit, Vector3};
#[macro_use]
extern crate approx;
use std::fs::File;
use std::io::prelude::*;

use parry3d::query::PointQuery;
use parry3d::shape::TriMesh;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use druid::widget::{Button, Flex, Label, TextBox};
use druid::{AppLauncher, commands, AppDelegate, Data, LocalizedString, PlatformError, Lens, Widget, WidgetExt, WindowDesc, Env, Target, Command, FontDescriptor, FontFamily, UnitPoint, DelegateCtx, FileDialogOptions, FileSpec,
    Handled,};


const FORWARD: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);
const BACKWARD: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
const RIGHT: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
const LEFT: Vector3<f32> = Vector3::new(-1.0, 0.0, 0.0);
const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
const DOWN: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);

const POINTS: [Vector3<f32>; 20] = [
    Vector3::new(-1.0, 1.0, -1.0),
    Vector3::new(0.0, 1.0, -1.0),
    Vector3::new(1.0, 1.0, -1.0),
    Vector3::new(-1.0, 1.0, 0.0),
    Vector3::new(1.0, 1.0, 0.0),
    Vector3::new(-1.0, 1.0, 1.0),
    Vector3::new(0.0, 1.0, 1.0),
    Vector3::new(1.0, 1.0, 1.0),
    Vector3::new(-1.0, 0.0, -1.0),
    Vector3::new(1.0, 0.0, -1.0),
    Vector3::new(-1.0, 0.0, 1.0),
    Vector3::new(1.0, 0.0, 1.0),
    Vector3::new(-1.0, -1.0, -1.0),
    Vector3::new(0.0, -1.0, -1.0),
    Vector3::new(1.0, -1.0, -1.0),
    Vector3::new(-1.0, -1.0, 0.0),
    Vector3::new(1.0, -1.0, 0.0),
    Vector3::new(-1.0, -1.0, 1.0),
    Vector3::new(0.0, -1.0, 1.0),
    Vector3::new(1.0, -1.0, 1.0),
];

#[derive(Clone, Data, Lens)]
struct AppState {
    input_file: String,
    output_file: String,
    block_size: String,
    large_grid: bool,
}

struct Delegate;

fn main() {


    let main_window = WindowDesc::new(ui_builder)
        .title("STL to Blueprint")
        .window_size((800.0, 800.0));

    // Data to be used in the app (=state)
    let data: AppState = AppState {
        input_file: format!("model.stl"),
        output_file: format!("bp.sbc"),
        block_size: format!("{}", 70),
        large_grid: true,
    };

    // Run the app
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .use_simple_logger() // Neat!
        .launch(data);

}


fn ui_builder() -> impl Widget<AppState> {

    let label = Label::new(|data: &AppState, _env: &Env| format!("Input File:")).padding(5.0);

    let textbox = TextBox::new()
        .with_placeholder("enter path to stl file")
        .with_text_size(18.0)
        .fix_width(550.0)
        .align_horizontal(UnitPoint::LEFT)
        .lens(AppState::input_file);
    //let label = Label::new(text).padding(5.0).center();

    // Two buttons with on_click callback
    /*
    let button_plus = Button::new("+1")
        .on_click(|_ctx, data: &mut AppState, _env| (*data).0 += 1)
        .padding(5.0);
    let button_minus = Button::new("-1")
        .on_click(|_ctx, data: &mut AppState, _env| (*data).0 -= 1)
        .padding(5.0);
*/


    let stl = FileSpec::new("STL model", &["stl"]);
    let read_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![stl])
        .default_name("test.stl")
        .name_label("Model")
        .title("Choose a model to import")
        .button_text("Import");
        
    let pick_button = Button::new("Pick").on_click(move |ctx, data: &mut AppState, _| {
        ctx.submit_command(Command::new(
            druid::commands::SHOW_OPEN_PANEL,
            read_dialog_options.clone(),
            Target::Auto,
        ))
    });

    let row1 = Flex::row()
        .with_child(label)
        .with_spacer(5.0)
        .with_child(textbox)
        .with_spacer(5.0)
        .with_child(pick_button)
        .padding(5.0);
    // Container for the two buttons
    /*
    let flex = Flex::row()
        .with_child(button_plus)
        .with_spacer(1.0)
        .with_child(button_minus);
        */

    // Container for the whole UI

    let label2 = Label::new(|data: &AppState, _env: &Env| format!("Output File:")).padding(5.0);

    let textbox2 = TextBox::new()
        .with_placeholder("enter path to output")
        .with_text_size(18.0)
        .fix_width(550.0)
        .align_horizontal(UnitPoint::LEFT)
        .lens(AppState::output_file);

    let row2 = Flex::row()
        .with_child(label2)
        .with_spacer(5.0)
        .with_child(textbox2)
        .padding(5.0);

    let label3 = Label::new(|data: &AppState, _env: &Env| format!("Largest Side in Blocks:")).padding(5.0);

    let textbox3 = TextBox::new()
        .with_placeholder("block count")
        .with_text_size(18.0)
        .fix_width(550.0)
        .align_horizontal(UnitPoint::LEFT)
        .lens(AppState::block_size);

    let row3 = Flex::row()
        .with_child(label3)
        .with_spacer(5.0)
        .with_child(textbox3)
        .padding(5.0);

    
    let label4 = Label::new(|data: &AppState, _env: &Env| format!("Using {} grid", if data.large_grid {"Large"} else {"Small"})).padding(5.0);
    
    let button_grid_size = Button::new("Swap Grid Size")
        .on_click(|_ctx, data: &mut AppState, _env| (*data).large_grid = !data.large_grid);

        /*
    let pick_read_file = Button::new("Choose")
        .on_click(|_ctx, data: &mut AppState, _env| (*data).large_grid = !data.large_grid)
        .padding(5.0);
        */


    let row4 = Flex::row()
        .with_child(label4)
        .with_spacer(5.0)
        .with_child(button_grid_size)
        .padding(5.0);

    let generate_button = Button::new("Generate!")
        .on_click(|_ctx, data: &mut AppState, _env| {
            convert_file_to_blueprint(data.block_size.parse::<usize>().unwrap(), data.input_file.clone(), data.large_grid, data.output_file.clone());
        })
        .padding(5.0);

    let row5 = Flex::row().with_child(generate_button);

    Flex::column().with_child(row1).with_child(row2).with_child(row3).with_child(row4).with_child(row5)
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {
            return Handled::Yes;
        }
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            data.input_file = file_info.path().to_str().unwrap().to_string();
            return Handled::Yes;
        }
        Handled::No
    }
}

fn convert_file_to_blueprint(blocksize: usize, mesh_path: String, large_grid: bool, output_file: String) {
    
    let mesh = parse_stl_to_mesh(mesh_path.as_str());

    let mut all_blocks = vec![
        CubeBlock::new([true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true], "LargeBlockArmorBlock", "SmallBlockArmorBlock"),
        CubeBlock::new([true,true,true,false,false,false,false,false,true,true,false,false,true,true,true,true,true,true,true,true], "LargeBlockArmorSlope", "SmallBlockArmorSlope"),
        CubeBlock::new([false,false,true,false,false,false,false,false,false,true,false,false,true,true,true,false,true,false,false,true], "LargeBlockArmorCorner", "SmallBlockArmorCorner"),
        CubeBlock::new([true,true,true,true,true,true,true,true,true,false,true,true,true,false,false,true,false,true,true,true], "LargeBlockArmorCornerInv", "SmallBlockArmorCornerInv"),
        CubeBlock::new([true,false,false,false,false,false,false,false,true,false,false,false,true,true,true,true,true,true,true,true], "LargeBlockArmorCornerSquare", "SmallBlockArmorCornerSquare"),
        CubeBlock::new([true,true,true,true,false,true,false,false,true,true,true,false,true,true,true,true,true,true,true,true], "LargeBlockArmorCornerSquareInverted", "SmallBlockArmorCornerSquareInverted"),
        CubeBlock::new([true,true,true,false,false,false,false,false,true,true,true,true,true,true,true,true,true,true,true,true], "LargeBlockArmorSlope2Base", "SmallBlockArmorSlope2Base"),
        CubeBlock::new([false,false,false,false,false,false,false,false,true,true,false,false,true,true,true,true,true,true,true,true], "LargeBlockArmorSlope2Tip", "SmallBlockArmorSlope2Tip"),
        CubeBlock::new([true,true,true,true,true,false,false,false,true,true,false,false,true,true,true,true,true,false,false,false], "LargeHalfArmorBlock", "HalfArmorBlock"),
        CubeBlock::new([false,false,false,false,false,false,false,false,true,true,false,false,true,true,true,true,true,false,false,false], "LargeHalfSlopeArmorBlock", "HalfSlopeArmorBlock"),
        CubeBlock::new([false,false,false,false,false,false,false,false,false,false,false,true,false,false,false,false,true,false,true,true], "LargeBlockArmorHalfSlopeCorner", "SmallBlockArmorHalfSlopeCorner"),
        CubeBlock::new([false,false,false,false,false,false,false,false,true,true,true,false,true,true,true,true,false,true,false,false], "LargeBlockArmorHalfCorner", "SmallBlockArmorHalfCorner"),
        CubeBlock::new([true,false,false,false,false,false,false,false,true,true,true,false,true,true,true,true,false,true,false,false], "LargeBlockArmorHalfSlopedCorner", "SmallBlockArmorHalfSlopedCorner"),
        CubeBlock::new([true,false,false,false,false,false,false,false,true,true,false,false,true,true,true,true,true,true,false,false], "LargeBlockArmorCorner2Base", "SmallBlockArmorCorner2Base"),
        CubeBlock::new([false,false,false,false,false,false,false,false,false,true,false,false,false,true,true,false,true,false,false,true], "LargeBlockArmorCorner2Tip", "SmallBlockArmorCorner2Tip"),
        CubeBlock::new([true,true,true,true,true,true,true,true,true,true,true,true,true,true,false,true,false,true,true,true], "LargeBlockArmorInvCorner2Base", "SmallBlockArmorInvCorner2Base"),
        CubeBlock::new([true,true,true,true,true,true,true,true,true,false,true,true,true,false,false,true,false,true,true,false], "LargeBlockArmorInvCorner2Tip", "SmallBlockArmorInvCorner2Tip"),
        CubeBlock::new([false,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true], "LargeBlockArmorHalfSlopeCornerInverted", "SmallBlockArmorHalfSlopeCornerInverted"),
        CubeBlock::new([true,true,false,true,false,true,true,false,true,true,true,true,true,true,true,true,true,true,true,true], "LargeBlockArmorHalfSlopeInverted", "SmallBlockArmorHalfSlopeInverted"),
        CubeBlock::new([false,false,false,false,false,false,false,false,false,false,true,false,true,false,false,true,false,true,true,true], "LargeBlockArmorSlopedCornerTip", "SmallBlockArmorSlopedCornerTip"),
        CubeBlock::new([true,true,true,false,true,false,false,true,true,true,true,true,true,true,true,true,true,true,true,true], "LargeBlockArmorSlopedCornerBase", "SmallBlockArmorSlopedCornerBase"),
        CubeBlock::new([true,false,false,false,false,false,false,false,true,true,true,false,true,true,true,true,true,true,true,true], "LargeBlockArmorSlopedCorner", "SmallBlockArmorSlopedCorner"),
        CubeBlock::new([false,false,false,false,false,false,false,false,true,true,true,false,true,true,true,true,true,true,true,true], "LargeBlockArmorHalfSlopedCornerBase", "SmallBlockArmorHalfSlopedCornerBase"),
    ];

    all_blocks.sort_by(|a, b| {
        b.Points
            .iter()
            .filter(|&x| *x)
            .count()
            .cmp(&a.Points.iter().filter(|&x| *x).count())
    });


    let mut grid = mesh_to_blocks(blocksize, large_grid, &mesh, &all_blocks);

    let mut changed = 1;
    //while changed > 0 {
    //}
    
    println!("beginning fill pass");
    changed = smooth_block_grid(blocksize, large_grid, &all_blocks, &mut grid, false);
    println!("completed fill pass with {} added blocks", changed);
    
    println!("beginning fill pass");
    changed = smooth_block_grid(blocksize, large_grid, &all_blocks, &mut grid, true);
    println!("completed fill pass with {} added blocks", changed);

    output_blocks_to_file(output_file.as_str(), &grid, large_grid);
}


fn parse_stl_to_mesh(path: &str) -> TriMesh {
    let mut file = OpenOptions::new().read(true).open(path).unwrap();
    let mut stl = stl_io::create_stl_reader(&mut file).unwrap();
    let triangles = stl.as_indexed_triangles().unwrap();
    let mut verts: Vec<na::Point<f32, 3_usize>> = Vec::new();
    let mut tris = Vec::new();
    for i in 0..triangles.vertices.len() {
        let vert = triangles.vertices[i];

        let x = vert[0];
        let y = vert[1];
        let z = vert[2];

        let pt = Point3::new(x, y, z);
        verts.push(pt);
    }

    for i in 0..triangles.faces.len() {
        let f = &triangles.faces[i];
        let pt = [
            f.vertices[0] as u32,
            f.vertices[1] as u32,
            f.vertices[2] as u32,
        ];
        tris.push(pt);
    }

    return TriMesh::new(verts, tris);
}

fn output_blocks_to_file(path: &str, grid: &Vec<Vec<Vec<Option<OrientedBlock>>>>, large_grid: bool) {
    let mut defs = Vec::new();

    for row in grid.iter() {
        for col in row.iter() {
            for block in col.iter() {
                match block {
                    Some(b) => defs.push(b.Xml.to_string()),
                    None => {}
                }
            }
        }
    }

    let mut block_file = File::create(path).unwrap();
    let _res = block_file.write_fmt(format_args!("<?xml version=\"1.0\"?>
    <Definitions xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">
      <ShipBlueprints>
        <ShipBlueprint xsi:type=\"MyObjectBuilder_ShipBlueprintDefinition\">
          <Id Type=\"MyObjectBuilder_ShipBlueprintDefinition\" Subtype=\"{name}\" />
          <DisplayName>{name}</DisplayName>
          <CubeGrids>
            <CubeGrid>
              <SubtypeName />
              <EntityId>134254513543793982</EntityId>
              <PersistentFlags>CastShadows InScene</PersistentFlags>
              <PositionAndOrientation>
                <Position x=\"0\" y=\"0\" z=\"0\" />
                <Forward x=\"0.0245060846\" y=\"-0.3717277\" z=\"-0.928018332\" />
                <Up x=\"0.669880331\" y=\"0.69516927\" z=\"-0.260768056\" />
                <Orientation>
                  <X>-0.172392666</X>
                  <Y>-0.07919351</Y>
                  <Z>-0.350280046</Z>
                  <W>0.9172312</W>
                </Orientation>
              </PositionAndOrientation>
              <LocalPositionAndOrientation xsi:nil=\"true\" />
              <GridSizeEnum>{size}</GridSizeEnum>
              <CubeBlocks>\n{}\n
              </CubeBlocks>
    
              <DisplayName>{name}</DisplayName>
              <DestructibleBlocks>true</DestructibleBlocks>
              <IsRespawnGrid>false</IsRespawnGrid>
              <LocalCoordSys>0</LocalCoordSys>
              <TargetingTargets />
            </CubeGrid>
          </CubeGrids>
          <EnvironmentType>None</EnvironmentType>
          <WorkshopId>0</WorkshopId>
          <OwnerSteamId>76561198116813162</OwnerSteamId>
          <Points>0</Points>
        </ShipBlueprint>
      </ShipBlueprints>
    </Definitions>",
        defs.join("\n"),
        name="Generated Test",
        size=if large_grid {"Large"} else {"Small"}
    ));
}

fn mesh_to_blocks<'a>(
    blocksize: usize,
    large_grid: bool,
    mesh: &TriMesh,
    avail_blocks: &'a Vec<CubeBlock<'a>>,
) -> Vec<Vec<Vec<Option<OrientedBlock<'a>>>>> {
    let aabb = mesh.local_aabb();

    println!("aabb min {}", aabb.mins);
    println!("aabb max {}", aabb.maxs);

    let mut biggest_dim = aabb.maxs.x - aabb.mins.x;
    biggest_dim = cmp::max(FloatOrd(biggest_dim), FloatOrd(aabb.maxs.y - aabb.mins.y)).0;
    biggest_dim = cmp::max(FloatOrd(biggest_dim), FloatOrd(aabb.maxs.z - aabb.mins.z)).0;
    let d = biggest_dim / blocksize as f32;

    let center = aabb.center();

    let half_extents = aabb.half_extents() + Vector3::new(1.5 * d, 1.5 * d, 1.5 * d);

    let aabb = parry3d::bounding_volume::AABB::from_half_extents(center, half_extents);

    let dist = (biggest_dim + (3.0 * d)) / blocksize as f32;
    let half_dist = dist / 2.0;

    let block_results: Vec<Vec<OrientedBlock>> = (0..blocksize)
        .into_par_iter()
        .map(|x| {
            let mut blocks = Vec::new();

            for y in 0..blocksize {
                for z in 0..blocksize {
                    let point = Point3::new(x as f32 * dist, y as f32 * dist, z as f32 * dist)
                        + aabb.mins.coords;
                    //check_block_space(point, &mesh, dist.0 * 1.1 / 2.0, &triangles);
                    if !aabb.contains_local_point(&point) {
                        continue;
                    }
                    let mut hits = Vec::new();
                    for pt in POINTS.iter() {
                        let contains =
                            check_block_space(&(point + (pt * half_dist)), &mesh, 0.95 * dist);
                        if contains {
                            hits.push(pt)
                        }
                    }

                    for block in avail_blocks.iter() {
                        let oris = block.get_orientation(&hits);
                        match oris {
                            Some((fwd, up, points)) => {
                                let block_def = format_block_def(
                                    if large_grid {
                                        block.LGSubtype
                                    } else {
                                        block.SGSubtype
                                    },
                                    x,
                                    y,
                                    z,
                                    fwd,
                                    up,
                                );

                                blocks.push(OrientedBlock::new(
                                    Point3::new(x, y, z),
                                    points,
                                    block_def,
                                    false
                                ));
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            return blocks;
        })
        .collect();


    let mut grid: Vec<Vec<Vec<Option<OrientedBlock>>>> =
        vec![vec![vec![None; blocksize as usize]; blocksize as usize]; blocksize as usize];

    for bs in block_results.iter() {
        for x in bs.iter() {
            grid[x.Pos.x][x.Pos.y][x.Pos.z] = Some(x.clone());
        }
    }

    return grid;
}

fn smooth_block_grid<'a>(
    blocksize: usize,
    large_grid: bool,
    avail_blocks: &'a Vec<CubeBlock<'a>>,
    grid: &mut Vec<Vec<Vec<Option<OrientedBlock<'a>>>>>,
    replace_existing: bool
) -> usize {
    let mut changed = 0;
    for block in avail_blocks.iter() {
        println!("{}", block.LGSubtype);
        let hit_count = block.Points.iter().filter(|&x| *x).count();
        let result: Vec<Vec<OrientedBlock>> = (0..blocksize)
            .into_par_iter()
            .map(|x| {
                let mut blocks = Vec::new();
                for y in 0..blocksize {
                    for z in 0..blocksize {
                        let p = Point3::new(x, y, z);
                        match &grid[x][y][z] {
                            Some(b) => {
                                if !replace_existing || b.Optim {
                                    continue;
                                }
                            }
                            _ => {}
                        }

                        let hits = fill_from_nearby_cubes(&p, &grid);
                        if hits.len() < hit_count {
                            continue;
                        }

                        let oris = block.get_orientation(&hits);
                        let mut found = false;
                        match oris {
                            Some((fwd, up, points)) => {
                                let block_def = format_block_def(
                                    if large_grid {
                                        block.LGSubtype
                                    } else {
                                        block.SGSubtype
                                    },
                                    p.x,
                                    p.y,
                                    p.z,
                                    fwd,
                                    up,
                                );

                                let b = OrientedBlock::new(
                                    Point3::new(p.x, p.y, p.z),
                                    points,
                                    block_def,
                                    true
                                );
                                blocks.push(b);
                                continue;
                            }
                            _ => {}
                        }
                        for subset in hits.clone().into_iter().combinations(hit_count) {
                            let oris = block.get_orientation(&subset);
                            match oris {
                                Some((fwd, up, points)) => {
                                    let block_def = format_block_def(
                                        if large_grid {
                                            block.LGSubtype
                                        } else {
                                            block.SGSubtype
                                        },
                                        p.x,
                                        p.y,
                                        p.z,
                                        fwd,
                                        up,
                                    );

                                    let b = OrientedBlock::new(
                                        Point3::new(p.x, p.y, p.z),
                                        points,
                                        block_def,
                                        true
                                    );
                                    blocks.push(b);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                return blocks;
            })
            .collect();
        for res in result {
            for block in res {
                grid[block.Pos.x][block.Pos.y][block.Pos.z] = Some(block.clone());
                changed = changed + 1;
            }
        }
    }
    return changed;
}

fn format_block_def(
    subtype: &str,
    x: usize,
    y: usize,
    z: usize,
    fwd: Direction,
    up: Direction,
) -> String {
    return format!(
        "            <MyObjectBuilder_CubeBlock xsi:type=\"MyObjectBuilder_CubeBlock\">
                <SubtypeName>{}</SubtypeName>
                <Min x=\"{}\" y=\"{}\" z=\"{}\" />
                <BlockOrientation Forward=\"{}\" Up=\"{}\" />
                <BuiltBy>0</BuiltBy>
            </MyObjectBuilder_CubeBlock>",
        subtype, x, y, z, fwd, up
    );
}

fn point_to_vertex(point: &na::Point3<f32>) -> stl_io::Vertex {
    return stl_io::Vertex::new([point.x, point.y, point.z]);
}

fn check_block_space(point: &Point3<f32>, mesh: &TriMesh, dist: f32) -> bool {
    let d = mesh.distance_to_local_point(point, true);
    return d < dist;
}

fn fill_from_nearby_cubes<'a>(
    pos: &Point3<usize>,
    grid: &Vec<Vec<Vec<Option<OrientedBlock>>>>,
) -> Vec<&'a Vector3<f32>> {
    let f_pos = Point3::new(pos.x as f32, pos.y as f32, pos.z as f32);
    let mut hits = Vec::new();

    for x in -1..2 {
        if (pos.x as i32) + x >= 0 && (pos.x as i32) + x < grid.len() as i32 {
            let col = &grid[(pos.x as i32 + x) as usize];
            for y in -1..2 {
                if (pos.y as i32) + y >= 0 && (pos.y as i32) + y < col.len() as i32 {
                    let row = &col[(pos.y as i32 + y) as usize];
                    for z in -1..2 {
                        if (pos.z as i32) + z >= 0 && (pos.z as i32) + z < row.len() as i32 {
                            match &row[(pos.z as i32 + z) as usize] {
                                Some(nbor) => {
                                    if pos == &nbor.Pos {
                                        continue;
                                    }
                                    for neigh_vert in nbor.Verts {
                                        let w_pos = (neigh_vert / 2.0)
                                            + Vector3::new(
                                                nbor.Pos.x as f32,
                                                nbor.Pos.y as f32,
                                                nbor.Pos.z as f32,
                                            );
                                        for pt in POINTS.iter() {
                                            let pt_w_pos = f_pos + (pt / 2.0);
                                            if relative_eq!(w_pos, pt_w_pos) {
                                                let hit =
                                                    hits.iter().find(|&x| relative_eq!(*x, pt));
                                                if hit.is_none() {
                                                    hits.push(pt);
                                                }
                                            }
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                }
            }
        }
    }
    return hits;
}

fn get_min_dist(min_dist: &mut FloatOrd<f32>, point: Point3<f32>, vertices: &Vec<stl_io::Vertex>) {
    for curr_vert in vertices {
        let x = curr_vert[0];
        let y = curr_vert[1];
        let z = curr_vert[2];
        let curr_point = Point3::new(x, y, z);
        let dist = FloatOrd(distance(&point, &curr_point));
        if (dist > FloatOrd(0.001)) {
            if dist < *min_dist {
                *min_dist = dist;
            }
        }
    }
}

fn get_vert_index(point: &Point3<f32>, verts: &Vec<na::Point3<f32>>) -> i32 {
    let mut curr_i = -1;
    for j in 0..verts.len() {
        let new_vert = verts[j];
        let d = distance_squared(&new_vert, &point);
        if d < 0.01 {
            curr_i = j as i32;
            break;
        }
    }
    return curr_i;
}

fn get_corrected_vert_index(
    index: usize,
    verts: &Vec<na::Point3<f32>>,
    triangles: &stl_io::IndexedMesh,
) -> u32 {
    return index as u32;

    let vert = triangles.vertices[index];
    let x = vert[0];
    let y = vert[1];
    let z = vert[2];
    let currPoint = Point3::new(x, y, z);
    let i = get_vert_index(&currPoint, verts);
    if (i != index as i32) {}
    return index as u32;
}

#[derive(EnumIter, Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    FORWARD,
    BACKWARD,
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Direction {
    fn opposite(&self) -> Direction {
        return match self {
            Direction::FORWARD => Direction::BACKWARD,
            Direction::BACKWARD => Direction::FORWARD,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
        };
    }

    fn get_rotation(fwd: &Direction, up: &Direction) -> Unit<na::Quaternion<f32>> {
        let fwd = match fwd {
            //Direction::LEFT => {Direction::RIGHT.get_vec()},
            //Direction::RIGHT => {Direction::LEFT.get_vec()},
            _ => fwd.get_vec(),
        };

        return Unit::face_towards(&fwd, &up.get_vec());
    }

    fn get_vec(&self) -> Vector3<f32> {
        return match self {
            Direction::FORWARD => FORWARD,
            Direction::BACKWARD => BACKWARD,
            Direction::LEFT => LEFT,
            Direction::RIGHT => RIGHT,
            Direction::UP => UP,
            Direction::DOWN => DOWN,
        };
    }

    fn from_vec(dir: &Vector3<f32>) -> Direction {
        //println!("{:?}", dir);
        if relative_eq!(dir, &FORWARD) {
            return Direction::FORWARD;
        };
        if relative_eq!(dir, &BACKWARD) {
            return Direction::BACKWARD;
        };
        if relative_eq!(dir, &LEFT) {
            return Direction::LEFT;
        };
        if relative_eq!(dir, &RIGHT) {
            return Direction::RIGHT;
        };
        if relative_eq!(dir, &UP) {
            return Direction::UP;
        };
        if relative_eq!(dir, &DOWN) {
            return Direction::DOWN;
        };
        panic!("Not a valid direction")
    }

    fn get_matrix_arr() -> [[(Direction, Direction, Unit<na::Quaternion<f32>>); 4]; 6] {
        let mut rows = Vec::new();
        for fwd in Direction::iter() {
            let mut col = Vec::new();
            for up in Direction::iter() {
                if up != fwd && up != fwd.opposite() {
                    //println!("Fwd={:?} Up={:?}", fwd, up);
                    col.push((fwd, up, Direction::get_rotation(&fwd, &up)));
                }
            }
            rows.push([col[0], col[1], col[2], col[3]])
        }
        return [rows[0], rows[1], rows[2], rows[3], rows[4], rows[5]];
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::FORWARD => "Forward",
                Direction::BACKWARD => "Backward",
                Direction::LEFT => "Left",
                Direction::RIGHT => "Right",
                Direction::UP => "Up",
                Direction::DOWN => "Down",
            }
        )
    }
}

struct Orientation {
    pub Fwd: Direction,
    pub Up: Direction,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Clone, Copy)]
enum Blocks {
    HalfSlope,
    FullBlock,
    CornerBlock,
    InvCorner,
    SquareCorner,
}

#[derive(Debug, Clone)]
struct OrientedBlock<'a> {
    pub Pos: Point3<usize>,
    pub Verts: &'a Vec<Point3<f32>>,
    pub Xml: String,
    pub Optim: bool,
}

impl<'a> OrientedBlock<'a> {
    pub fn new(pos: Point3<usize>, verts: &'a Vec<Point3<f32>>, xml: String, optim: bool) -> OrientedBlock<'a> {
        return OrientedBlock {
            Pos: pos,
            Verts: verts,
            Xml: xml,
            Optim: optim
        };
    }
}

struct CubeBlock<'a> {
    pub Points: [bool; 20],
    pub LGSubtype: &'a str,
    pub SGSubtype: &'a str,
    pub Orientations: Vec<(Direction, Direction, Vec<Point3<f32>>)>,
}

impl<'a> CubeBlock<'a> {
    pub fn new(
        points: [bool; 20],
        large_subtype: &'a str,
        small_subtype: &'a str,
    ) -> CubeBlock<'a> {
        let mut orientations = Vec::new();
        let mut orig_pts = Vec::new();
        let lookup = Direction::get_matrix_arr();
        for i in 0..points.len() {
            let pt = points[i];
            if pt {
                let vec = POINTS[i];
                orig_pts.push(Point3::from(vec));
            }
        }

        println!("{}", large_subtype);
        for directions in lookup.iter() {
            for (fwd, up, quat) in directions {
                let mut pts = Vec::new();
                //let mut strs = Vec::new();
                for pt in orig_pts.iter() {
                    let pt = quat.transform_point(&pt);
                    pts.push(pt);
                    //strs.push(format!("{}{}{}", if pt.x == -1.0 { "L" } else { "R" }, if pt.y == -1.0 { "D" } else { "U" }, if pt.z == -1.0 { "F" } else { "B" }));
                    //println!("{:?}", pt);
                }
                //println!("F={} U={} pts={:?} RF={} RU={}", fwd, up, strs, Direction::from_vec(&(quat * FORWARD)), Direction::from_vec(&(quat * UP)));
                orientations.push((
                    Direction::from_vec(&(quat * FORWARD)),
                    Direction::from_vec(&(quat * UP)),
                    pts,
                ));
            }
        }

        return CubeBlock {
            Points: points,
            LGSubtype: large_subtype,
            SGSubtype: small_subtype,
            Orientations: orientations,
        };
    }

    pub fn get_orientation(
        &self,
        points: &Vec<&Vector3<f32>>,
    ) -> Option<(Direction, Direction, &Vec<Point3<f32>>)> {
        for (fwd, up, oriented_points) in self.Orientations.iter() {
            if points.len() != oriented_points.len() {
                return None;
            }
            for i in 0..points.len() {
                let vec = points[i];
                let p = Point3::new(vec.x, vec.y, vec.z);
                let x = oriented_points.iter().find(|&x| relative_eq!(*x, p));
                if x.is_some() {
                    if i == points.len() - 1 {
                        return Some((*fwd, *up, oriented_points));
                    }
                } else {
                    break;
                }
            }
        }
        return None;
    }
}

/*
let params = vhacd::VHACDParameters {
    concavity: 0.01,
    alpha: 0.05,
    beta: 0.05,
    resolution: 128,
    plane_downsampling: 4,
    convex_hull_downsampling: 4,
    fill_mode: transformation::voxelization::FillMode::FloodFill {
        detect_cavities: false
    },
    convex_hull_approximation: false,
    max_convex_hulls: 2048
};
let lookup = Direction::get_matrix_arr();


println!("starting complex decomposition");
let shared = SharedShape::convex_decomposition_with_params(&verts, &tris, &params);
println!("finished complex decomposition");
let mesh = shared.as_compound().unwrap();
println!("merging aabbs: {}", mesh.aabbs().len());
let mut aabb = aabb::AABB::new_invalid();
for x in mesh.aabbs() {
    aabb.merge(&x);
}

let mut stlMesh = Vec::new();



let shapes = mesh.shapes();
for (iso, shape) in shapes {
    let polys = shape.as_convex_polyhedron();
    match polys {
        Some(polys) => {
            let (verts, faces) = polys.to_trimesh();

            println!("verts={} faces={}", verts.len(), faces.len());
            for [x, y, z] in faces {
                match ccw_face_normal([&verts[x as usize], &verts[y as usize], &verts[z as usize]]) {
                    Some(normal) => {
                        stlMesh.push(stl_io::Triangle {
                            normal: stl_io::Normal::new([normal.x, normal.y, normal.z]),
                            vertices: [point_to_vertex(&verts[x as usize]), point_to_vertex(&verts[y as usize]), point_to_vertex(&verts[z as usize])]
                        })
                    }
                    None => {
                        stlMesh.push(stl_io::Triangle {
                            normal: stl_io::Normal::new([1.0, 0.0, 0.0]),
                            vertices: [point_to_vertex(&verts[x as usize]), point_to_vertex(&verts[y as usize]), point_to_vertex(&verts[z as usize])]
                        })
                    }
                }
            }
        },
        None => {
            println!("Found non polyhedron {:?}", shape.shape_type());
        }
    }
}

let mut file = OpenOptions::new().write(true).create(true).truncate(true).open("mesh.stl").unwrap();
stl_io::write_stl(&mut file, stlMesh.iter()).unwrap();
*/

/*
return match (fwd, up) {
    (Direction::FORWARD, Direction::UP) => Matrix4::from_euler_angles(0 as f32, 0 as f32, 0 as f32),
    (Direction::FORWARD, Direction::DOWN) => Matrix4::from_euler_angles((180 as f32).to_radians(), 0 as f32, 0 as f32),
    (Direction::FORWARD, Direction::LEFT) => Matrix4::from_euler_angles((-90 as f32).to_radians(), 0 as f32, 0 as f32),
    (Direction::FORWARD, Direction::RIGHT) => Matrix4::from_euler_angles((90 as f32).to_radians(), 0 as f32, 0 as f32),
    (Direction::BACKWARD, Direction::UP) => Matrix4::from_euler_angles(0 as f32, 0 as f32, 0 as f32),
    (Direction::BACKWARD, Direction::DOWN) => Matrix4::from_euler_angles((180 as f32).to_radians(), 0 as f32, (180 as f32).to_radians()),
    (Direction::BACKWARD, Direction::LEFT) => Matrix4::from_euler_angles((-90 as f32).to_radians(), 0 as f32, (180 as f32).to_radians()),
    (Direction::BACKWARD, Direction::RIGHT) => Matrix4::from_euler_angles((90 as f32).to_radians(), 0 as f32, (180 as f32).to_radians()),
    (Direction::RIGHT, Direction::UP) => Matrix4::from_euler_angles(0 as f32, 0 as f32, (90 as f32).to_radians()),
    (Direction::RIGHT, Direction::DOWN) => Matrix4::from_euler_angles((180 as f32).to_radians(), 0 as f32, (-90 as f32).to_radians()),
    (Direction::RIGHT, Direction::FORWARD) => Matrix4::from_euler_angles(0 as f32, (-90 as f32).to_radians(), (90 as f32).to_radians()),
    (Direction::RIGHT, Direction::BACKWARD) => Matrix4::from_euler_angles(0 as f32, (90 as f32).to_radians(), (-90 as f32).to_radians()),
    (Direction::LEFT, Direction::UP) => Matrix4::from_euler_angles(0 as f32, 0 as f32, (-90 as f32).to_radians()),
    (Direction::LEFT, Direction::DOWN) => Matrix4::from_euler_angles((180 as f32).to_radians(), 0 as f32, (90 as f32).to_radians()),
    (Direction::LEFT, Direction::FORWARD) => Matrix4::from_euler_angles(0 as f32, (-90 as f32).to_radians(), (-90 as f32).to_radians()),
    (Direction::LEFT, Direction::BACKWARD) => Matrix4::from_euler_angles(0 as f32, (90 as f32).to_radians(), (90 as f32).to_radians()),
    (Direction::UP, Direction::LEFT) => Matrix4::from_euler_angles((-90 as f32).to_radians(), 0 as f32, (90 as f32).to_radians()),
    (Direction::UP, Direction::RIGHT) => Matrix4::from_euler_angles((90 as f32).to_radians(), 0 as f32, (-90 as f32).to_radians()),
    (Direction::UP, Direction::FORWARD) => Matrix4::from_euler_angles(0 as f32, (-90 as f32).to_radians(), (180 as f32).to_radians()),
    (Direction::UP, Direction::BACKWARD) => Matrix4::from_euler_angles(0 as f32, (90 as f32).to_radians(), (180 as f32).to_radians()),
    (Direction::DOWN, Direction::LEFT) => Matrix4::from_euler_angles((-90 as f32).to_radians(), 0 as f32, (-90 as f32).to_radians()),
    (Direction::DOWN, Direction::RIGHT) => Matrix4::from_euler_angles((90 as f32).to_radians(), 0 as f32, (90 as f32).to_radians()),
    (Direction::DOWN, Direction::FORWARD) => Matrix4::from_euler_angles(0 as f32, (-90 as f32).to_radians(), 0 as f32),
    (Direction::DOWN, Direction::BACKWARD) => Matrix4::from_euler_angles(0 as f32, (90 as f32).to_radians(), 0 as f32),
    _ => Matrix4::from_euler_angles(0 as f32, 0 as f32, 0 as f32)
};
*/

/*
if below.is_some()
{
    let x = below.unwrap();
    x.d;
    if relative_eq!(x.normal, DOWN) {
        println!("Got a down normal on an up cast");
    }
    if relative_eq!(x.normal, UP) {
        println!("Got an up normal on an up cast");
    }
    match x.feature {
        FeatureId::Vertex(x) => {

        },
        FeatureId::Edge(x) => {

        },
        FeatureId::Face(x) => {
            //let f = &triangles.faces[x as usize];
            //println!("Face normal of {},{},{}", f.normal[0],f.normal[1],f.normal[2]);
        },
        FeatureId::Unknown => {

        }
    }

}


if below.is_some() || above.is_some()  || left_of.is_some()  || right_of.is_some()  || behind.is_some()  || in_front.is_some()  {
    /*println!("below={} above={} left_of={} right_of={} behind={} in_front={}",
        below.is_some(),
        above.is_some(),
        left_of.is_some(),
        right_of.is_some(),
        behind.is_some(),
        in_front.is_some());*/
}
*/
