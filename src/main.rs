use float_ord::FloatOrd;
use std::cmp;
use rayon::prelude::*;
use std::fmt;
use std::fs::OpenOptions;
extern crate nalgebra as na;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use serde_xml_rs::from_reader;
use serde_xml_rs::to_string;

use na::{distance, distance_squared, Isometry3, Point3, Vector3, Matrix4, Rotation3, Unit};
#[macro_use]
extern crate approx;
use std::fs::File;
use std::io::prelude::*;

use parry3d::math;
use parry3d::utils::ccw_face_normal;
use parry3d::query;
use parry3d::query::PointQuery;
use parry3d::query::Ray;
use parry3d::query::RayCast;
use parry3d::shape::Cuboid;
use parry3d::shape::Compound;
use parry3d::shape::SharedShape;
use parry3d::shape::FeatureId;
use parry3d::shape::TriMesh;
use parry3d::bounding_volume::aabb;
use parry3d::bounding_volume::BoundingVolume;
use parry3d::transformation;
use parry3d::transformation::vhacd;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;

const FORWARD: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);
const BACKWARD: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
const RIGHT: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
const LEFT: Vector3<f32> = Vector3::new(-1.0, 0.0, 0.0);
const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
const DOWN: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);

const POINTS: [ Vector3<f32> ; 8] = [
    Vector3::new(-1.0,  1.0, -1.0),
    Vector3::new( 1.0,  1.0, -1.0),
    Vector3::new(-1.0,  1.0,  1.0),
    Vector3::new( 1.0,  1.0,  1.0),
    Vector3::new(-1.0, -1.0, -1.0),
    Vector3::new( 1.0, -1.0, -1.0),
    Vector3::new(-1.0, -1.0,  1.0),
    Vector3::new( 1.0, -1.0,  1.0),
];

fn main() {


    let HalfSlope: CubeBlock = CubeBlock::new([true, true, false, false, true, true, true, true], "LargeBlockArmorSlope");

    let FullBlock: CubeBlock = CubeBlock::new([true, true, true, true, true, true, true, true], "LargeHeavyBlockArmorBlock");
    
    let CornerBlock: CubeBlock = CubeBlock::new([false, true, false, false, true, true, false, true], "LargeBlockArmorCorner");
    
    let InvCorner: CubeBlock = CubeBlock::new([true, true, true, true, true, false, true, true], "LargeBlockArmorCornerInv");
    
    let SquareCorner: CubeBlock = CubeBlock::new([true, false, false, false, true, true, true, true], "LargeBlockArmorCornerSquare");



    let blocksize = 101;

    let mut file = OpenOptions::new().read(true).open("falcon.stl").unwrap();
    let mut stl = stl_io::create_stl_reader(&mut file).unwrap();
    let triangles = stl.as_indexed_triangles().unwrap();
    let size_hint = stl.size_hint();
    let mut verts: Vec<na::Point<f32, 3_usize>> = Vec::new();
    let mut tris = Vec::new();
    for i in 0..triangles.vertices.len() {
        let vert = triangles.vertices[i];

        let x = FloatOrd(vert[0]);
        let y = FloatOrd(vert[1]);
        let z = FloatOrd(vert[2]);

        let pt = Point3::new(x.0, y.0, z.0);

        verts.push(pt);
        /*
        let mut currIndex = get_vert_index(&pt, &verts);
        if currIndex == -1 {
            verts.push(pt);
            currIndex = (verts.len() - 1) as i32;
        } else {

        }
        */
        //getMinDistance(&mut dist, pt, &triangles.vertices);
    }

    for i in 0 .. triangles.faces.len() {
        let f = &triangles.faces[i];
        let pt = [
            get_corrected_vert_index(f.vertices[0], &verts, &triangles),
            get_corrected_vert_index(f.vertices[1], &verts, &triangles),
            get_corrected_vert_index(f.vertices[2], &verts, &triangles)];
        tris.push(pt);
    }


    let mut mesh = TriMesh::new(verts, tris);
    let aabb = mesh.local_aabb();

    println!("aabb min {}", aabb.mins);
    println!("aabb max {}", aabb.maxs);
    


    let mut biggestDim = aabb.maxs.x - aabb.mins.x;
    biggestDim = cmp::max(FloatOrd(biggestDim), FloatOrd(aabb.maxs.y - aabb.mins.y)).0;
    biggestDim = cmp::max(FloatOrd(biggestDim), FloatOrd(aabb.maxs.z - aabb.mins.z)).0;
    let dist = biggestDim / blocksize as f32;
    let mut inside = 0;
    let mut outside = 0;
    let halfDist = dist / 2.0;
    let mut x = aabb.mins.x + (dist / 2.0);
    
    


    let all_blocks: Vec<Vec<String>> = (0.. blocksize).into_par_iter().map(|x|  {
        let mut blocks = Vec::new();
        for y in 0 .. blocksize {
            for z in 0 .. blocksize {
                let point = Point3::new(x as f32 * dist, y as f32 * dist, z as f32 * dist) + aabb.mins.coords;
                //check_block_space(point, &mesh, dist.0 * 1.1 / 2.0, &triangles);
                if !aabb.contains_local_point(&point) {
                    continue;
                }
                let mut hits = Vec::new();
                for pt in POINTS.iter() {
                    let contains = check_block_space(&(point + (pt * halfDist)), &mesh, 1.0 * dist);
                    if contains {
                        hits.push(pt)
                    }
                }
                let block = CubeBlock::get_block(&hits);

                match block {
                    Some(block) => {
                        //inside = inside + 1;
                        let cube = match block {
                            Blocks::HalfSlope => &HalfSlope,
                            Blocks::CornerBlock => &CornerBlock,
                            Blocks::InvCorner => &InvCorner,
                            Blocks::SquareCorner => &SquareCorner,
                            Blocks::FullBlock => &FullBlock,
                        };
                        if block == Blocks::FullBlock {
                            let block_def = format!("            <MyObjectBuilder_CubeBlock xsi:type=\"MyObjectBuilder_CubeBlock\">
                            <SubtypeName>{}</SubtypeName>
                            <Min x=\"{}\" y=\"{}\" z=\"{}\" />
                            <BuiltBy>144115188075855895</BuiltBy>
                        </MyObjectBuilder_CubeBlock>", cube.Subtype, x, y, z);
                            blocks.push(block_def);
                        } else {
                            let oris = cube.get_orientation(&hits);
                            match oris {
                                Some((fwd, up)) => {
                                    let block_def = format!("            <MyObjectBuilder_CubeBlock xsi:type=\"MyObjectBuilder_CubeBlock\">
                                    <SubtypeName>{}</SubtypeName>
                                    <Min x=\"{}\" y=\"{}\" z=\"{}\" />
                                    <BlockOrientation Forward=\"{}\" Up=\"{}\" />
                                    <BuiltBy>144115188075855895</BuiltBy>
                                </MyObjectBuilder_CubeBlock>", cube.Subtype, x, y, z, fwd, up);
                                    blocks.push(block_def);
                                }
                                _ => {

                                }
                            }
                        }
                    },
                    None => {
                        //outside = outside + 1;
                    }
                }
            }
        }
        return blocks;
    }).collect();

    let mut blocks = Vec::new();

    for b in all_blocks {
        for x in b {
            blocks.push(x);
        }
    }
    

    let mut blockFile = File::create("blocks.txt").unwrap();
    blockFile.write_fmt(format_args!("          <CubeBlocks>\n{}\n          </CubeBlocks>", blocks.join("\n")));

    println!("inside={} outside={} blocks={}", inside, outside, blocks.len());
    let ray = Ray::new(Point3::new(-50.0, 0.0, 0.0), RIGHT);

    let intersects_ray = mesh.intersects_ray(&Isometry3::identity(), &ray, 5000.0);
    println!("{}", intersects_ray);
}

fn point_to_vertex(point: &na::Point3<f32>) -> stl_io::Vertex
{
    return stl_io::Vertex::new([point.x, point.y, point.z]);
}


fn check_block_space(point: &Point3<f32>, mesh: &TriMesh, dist: f32) -> bool
{

    let d = mesh.distance_to_local_point(point, true);
    return d < dist;
    /*

    let mut results = Vec::new();

    let ray = Ray::new(*point, UP);
    results.push(mesh.cast_local_ray_and_get_normal(&ray, dist, true));
    let ray = Ray::new(*point, DOWN);
    results.push(mesh.cast_local_ray_and_get_normal(&ray, dist, true));
    let ray = Ray::new(*point, RIGHT);
    results.push(mesh.cast_local_ray_and_get_normal(&ray, dist, true));
    let ray = Ray::new(*point, LEFT);
    results.push(mesh.cast_local_ray_and_get_normal(&ray, dist, true));
    let ray = Ray::new(*point, FORWARD);
    results.push(mesh.cast_local_ray_and_get_normal(&ray, dist, true));
    let ray = Ray::new(*point, BACKWARD);
    results.push(mesh.cast_local_ray_and_get_normal(&ray, dist, true));

    for res in results {
        match res {
            Some(x) => {
                return true;
            },
            None => {
            }
        }
    };

    return false;
    */

}


fn getMinDistance(minDist: &mut FloatOrd<f32>, point: Point3<f32>, vertices: &Vec<stl_io::Vertex>) {
    for currVert in vertices {
        let x = currVert[0];
        let y = currVert[1];
        let z = currVert[2];
        let currPoint = Point3::new(x, y, z);
        let dist = FloatOrd(distance(&point, &currPoint));
        if (dist > FloatOrd(0.001)) {
            if (dist < *minDist) {
                *minDist = dist;
            }
        }
    }
}

fn get_vert_index(point: &Point3<f32>, verts: &Vec<na::Point<f32, 3_usize>>) -> i32 {
    let mut currIndex = -1;
    for j in 0..verts.len() {
        let newVert = verts[j];
        let d = distance_squared(&newVert, &point);
        if d < 0.01 {
            currIndex = j as i32;
            break;
        }
    }
    return currIndex;
}

fn get_corrected_vert_index(index: usize, verts: &Vec<na::Point<f32, 3_usize>>, triangles: &stl_io::IndexedMesh) -> u32
{
    return index as u32;

    let vert = triangles.vertices[index];
    let x = vert[0];
    let y = vert[1];
    let z = vert[2];
    let currPoint = Point3::new(x, y, z);
    let i = get_vert_index(&currPoint, verts);
    if (i != index as i32) {
    }
    return index as u32;
}

fn testSerde() {
    let mut bps = Vec::new();
    bps.push(ShipBlueprint {
        name: "Test".to_string(),
        xsi: "MyObjectBuilder_ShipBlueprintDefinition".to_string(),
        id: GridId {
            typeId: "MyObjectBuilder_ShipBlueprintDefinition".to_string(),
            subtypeId: "Test Blueprint".to_string(),
        },
    });

    let defs = Definitions {
        ShipBlueprints: bps,
    };
    let serialized = to_string(&defs).unwrap();
    println!("{}", serialized);
}


#[derive(EnumIter, Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    FORWARD,
    BACKWARD,
    UP,
    DOWN,
    LEFT,
    RIGHT
}

impl Direction {
    fn opposite(&self) -> Direction
    {
        return match self {
            Direction::FORWARD => Direction::BACKWARD,
            Direction::BACKWARD => Direction::FORWARD,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
        }
    }

    fn get_rotation(fwd: &Direction, up: &Direction) -> Unit<na::Quaternion<f32>>
    {

        let fwd = match fwd {
            //Direction::LEFT => {Direction::RIGHT.get_vec()},
            //Direction::RIGHT => {Direction::LEFT.get_vec()},
            _ => {fwd.get_vec()}
        };

       return Unit::face_towards(&fwd, &up.get_vec());
    }

    
    fn get_vec(&self) -> Vector3<f32>
    {
        return match self {
            Direction::FORWARD => FORWARD,
            Direction::BACKWARD =>  BACKWARD,
            Direction::LEFT => LEFT,
            Direction::RIGHT => RIGHT,
            Direction::UP => UP,
            Direction::DOWN => DOWN,
        }
    }

    fn from_vec(dir: &Vector3<f32>) -> Direction
    {
        println!("{:?}", dir);
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

    fn get_matrix_arr() -> [[(Direction, Direction, Unit<na::Quaternion<f32>>) ; 4] ; 6] {
        let mut rows = Vec::new();
        for fwd in Direction::iter() {
            let mut col = Vec::new();
            for up in Direction::iter() {
                if up != fwd && up != fwd.opposite()
                {
                    //println!("Fwd={:?} Up={:?}", fwd, up);
                    col.push((fwd, up, Direction::get_rotation(&fwd, &up)));
                }
            }
            rows.push([
                col[0],
                col[1],
                col[2],
                col[3]
            ])
        }
        return [
            rows[0],
            rows[1],
            rows[2],
            rows[3],
            rows[4],
            rows[5]
        ]
    }
}


impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Direction::FORWARD => "Forward",
            Direction::BACKWARD => "Backward",
            Direction::LEFT => "Left",
            Direction::RIGHT => "Right",
            Direction::UP => "Up",
            Direction::DOWN => "Down",
        })
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
    SquareCorner
}

struct CubeBlock<'a> {
    pub Points: [bool; 8],
    pub Subtype: &'a str,
    pub Orientations: Vec<(Direction, Direction, Vec<Point3<f32>>)>
}

impl<'a> CubeBlock<'a> {

    pub fn new(points: [bool; 8], subtype: &'a str) -> CubeBlock {
        let mut orientations = Vec::new();
        let mut orig_pts = Vec::new();
        let lookup = Direction::get_matrix_arr();
        for i in 0 .. points.len() {
            let pt = points[i];
            if pt {
                let vec = POINTS[i];
                orig_pts.push(Point3::from_coordinates(vec));
            }
        }

        println!("{}", subtype);
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
                orientations.push((Direction::from_vec(&(quat * FORWARD)), Direction::from_vec(&(quat * UP)), pts));
            }
        }

        return CubeBlock {
            Points: points,
            Subtype: subtype,
            Orientations: orientations
        }

    }


    pub fn get_block(points: &Vec<&Vector3<f32>>) -> Option<Blocks> {
        return match points.len() {
            8 => Some(Blocks::FullBlock),
            7 => Some(Blocks::InvCorner),
            6 => Some(Blocks::HalfSlope),
            5 => Some(Blocks::SquareCorner),
            4 => Some(Blocks::CornerBlock),
            _ => None
        }
        
    }

    pub fn get_orientation(&self, points: &Vec<&Vector3<f32>>) -> Option<(Direction, Direction)> {
        for (fwd, up, oriented_points) in self.Orientations.iter() {
            for i in 0 .. points.len() {
                let vec = points[i];
                let p = Point3::new(vec.x, vec.y, vec.z);
                let x = oriented_points.iter().find(|&x| relative_eq!(x, &p));
                if x.is_some() {
                    if (i == points.len() - 1) {
                        return Some((*fwd, *up));
                    }
                } else {
                    break;
                }
            }
        }
        return None;
    }
}


#[derive(Debug, Deserialize, Serialize)]
struct Definitions {
    #[serde(rename = "ShipBlueprint", default)]
    pub ShipBlueprints: Vec<ShipBlueprint>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ShipBlueprint {
    #[serde(rename = "Id")]
    pub id: GridId,
    #[serde(rename = "DisplayName")]
    pub name: String,

    #[serde(rename = "xsi:type")]
    pub xsi: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GridId {
    #[serde(rename = "Type")]
    pub typeId: String,

    #[serde(rename = "Subtype")]
    pub subtypeId: String,
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
