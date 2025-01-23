//! A simple script that generates a Blender scene from a Raydar scene.
//!
//! This is used for debugging and testing rendering results, in comparison
//! to Blender Cycles.
//!
//! It uses Blender's Python API to create the scene and relies on a `blender`
//! binary, available in `$PATH`.
//!
use cgmath::Point3;
use raydar::scene::{
    material::Material,
    objects::{Cube, Geometry, Object, Sphere},
    world::World,
    Scene,
};
use std::{fs::File, io::Write, path::PathBuf};

// Transform the basis of the coordinate system to be compatible with Blender
fn convert_point(p: Point3<f32>) -> Point3<f32> {
    // In Blender: X right, Y forward, Z up
    // In Raydar: X right, Z forward, Y up
    Point3::new(p.x, -p.z, p.y)
}

fn generate_material_setup(material: &Material) -> String {
    format!(
        r#"
    mat = bpy.data.materials.new(name="Material")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    
    # Clear default nodes
    nodes.clear()
    
    # Create principled BSDF
    principled = nodes.new('ShaderNodeBsdfPrincipled')
    principled.inputs['Base Color'].default_value = [{}, {}, {}, 1.0]
    principled.inputs['Metallic'].default_value = {}
    principled.inputs['Roughness'].default_value = {}
    principled.inputs['IOR'].default_value = {}
    principled.inputs['Transmission Weight'].default_value = {}
    principled.inputs['Emission Color'].default_value = [{}, {}, {}, 1.0]
    principled.inputs['Emission Strength'].default_value = {}
    
    # Create output node
    output = nodes.new('ShaderNodeOutputMaterial')
    
    # Link nodes
    links.new(principled.outputs['BSDF'], output.inputs['Surface'])
"#,
        material.albedo.x,
        material.albedo.y,
        material.albedo.z,
        material.metallic,
        material.roughness,
        material.ior,
        material.transmission,
        material.emission_color.x,
        material.emission_color.y,
        material.emission_color.z,
        material.emission_strength,
    )
}

fn generate_object_setup(object: &Object, index: usize) -> String {
    let (mesh_type, transform) = match &object.geometry {
        Geometry::Sphere(Sphere { center, radius }) => {
            let blender_center = convert_point(*center);
            (
                "bpy.ops.mesh.primitive_uv_sphere_add()",
                format!(
                    r#"
    obj.location = [{}, {}, {}]
    obj.scale = [{}, {}, {}]
    # Set smooth shading
    for polygon in obj.data.polygons:
        polygon.use_smooth = True"#,
                    blender_center.x, blender_center.y, blender_center.z, radius, radius, radius
                ),
            )
        }
        Geometry::Cube(Cube {
            center,
            side_length,
        }) => {
            let blender_center = convert_point(*center);
            (
                "bpy.ops.mesh.primitive_cube_add()",
                format!(
                    r#"
    obj.location = [{}, {}, {}]
    obj.scale = [{}, {}, {}]"#,
                    blender_center.x,
                    blender_center.y,
                    blender_center.z,
                    side_length / 2.0,
                    side_length / 2.0,
                    side_length / 2.0
                ),
            )
        }
    };

    format!(
        r#"
    # Create object {}
    {}
    obj = bpy.context.active_object
    {}
    
    # Assign material
    if obj.data.materials:
        obj.data.materials[0] = mat
    else:
        obj.data.materials.append(mat)
"#,
        index, mesh_type, transform
    )
}

fn generate_camera_setup(scene: &Scene) -> String {
    let camera_pos = convert_point(scene.camera.position());
    let camera_target = convert_point(scene.camera.target());

    format!(
        r#"
    # Setup camera
    bpy.ops.object.camera_add()
    camera = bpy.context.active_object
    camera.location = [{}, {}, {}]
    
    # Create empty as target
    bpy.ops.object.empty_add(type='PLAIN_AXES', location=({}, {}, {}))
    target = bpy.context.active_object
    
    # Add Track To constraint
    constraint = camera.constraints.new('TRACK_TO')
    constraint.target = target
    constraint.track_axis = 'TRACK_NEGATIVE_Z'
    constraint.up_axis = 'UP_Y'
    
    # Set camera as active
    bpy.context.scene.camera = camera
"#,
        camera_pos.x, camera_pos.y, camera_pos.z, camera_target.x, camera_target.y, camera_target.z,
    )
}

fn generate_world_setup(world: &World) -> String {
    match world {
        World::SolidColor(color) => format!(
            r#"
    # Setup world background color
    world = bpy.context.scene.world
    if not world:
        world = bpy.data.worlds.new("World")
        bpy.context.scene.world = world
    
    world.use_nodes = True
    nodes = world.node_tree.nodes
    links = world.node_tree.links
    
    # Clear default nodes
    nodes.clear()
    
    # Create background node
    background = nodes.new('ShaderNodeBackground')
    background.inputs['Color'].default_value = [{}, {}, {}, 1.0]
    
    # Create output node
    output = nodes.new('ShaderNodeOutputWorld')
    
    # Link nodes
    links.new(background.outputs['Background'], output.inputs['Surface'])
"#,
            color.x, color.y, color.z
        ),
        World::SkyColor {
            top_color,
            bottom_color,
        } => format!(
            r#"
    # Setup sky gradient
    world = bpy.context.scene.world
    if not world:
        world = bpy.data.worlds.new("World")
        bpy.context.scene.world = world
    
    world.use_nodes = True
    nodes = world.node_tree.nodes
    links = world.node_tree.links
    
    # Clear default nodes
    nodes.clear()
    
    # Create nodes
    tex_coord = nodes.new('ShaderNodeTexCoord')
    separate = nodes.new('ShaderNodeSeparateXYZ')
    add = nodes.new('ShaderNodeMath')
    multiply = nodes.new('ShaderNodeMath')
    mapping = nodes.new('ShaderNodeMapping')
    gradient = nodes.new('ShaderNodeTexGradient')
    color_ramp = nodes.new('ShaderNodeValToRGB')
    background = nodes.new('ShaderNodeBackground')
    output = nodes.new('ShaderNodeOutputWorld')
    
    # Position nodes for better organization
    tex_coord.location = (-1100, 0)
    separate.location = (-900, 0)
    add.location = (-700, 0)
    multiply.location = (-500, 0)
    mapping.location = (-300, 0)
    gradient.location = (-100, 0)
    color_ramp.location = (100, 0)
    background.location = (300, 0)
    output.location = (500, 0)
    
    # Setup nodes
    add.operation = 'ADD'
    add.inputs[1].default_value = 1.0
    
    multiply.operation = 'MULTIPLY'
    multiply.inputs[1].default_value = 0.5
    
    gradient.gradient_type = 'LINEAR'
    
    # Setup color ramp
    color_ramp.color_ramp.interpolation = 'LINEAR'
    color_ramp.color_ramp.elements[0].position = 0.0
    color_ramp.color_ramp.elements[0].color = [{}, {}, {}, 1.0]  # Bottom color
    color_ramp.color_ramp.elements[1].position = 1.0
    color_ramp.color_ramp.elements[1].color = [{}, {}, {}, 1.0]  # Top color
    
    # Link nodes
    links.new(tex_coord.outputs['Generated'], separate.inputs['Vector'])
    links.new(separate.outputs['Y'], add.inputs[0])
    links.new(add.outputs[0], multiply.inputs[0])
    links.new(multiply.outputs[0], color_ramp.inputs['Fac'])
    links.new(color_ramp.outputs['Color'], background.inputs['Color'])
    links.new(background.outputs['Background'], output.inputs['Surface'])
"#,
            bottom_color.x, bottom_color.y, bottom_color.z, top_color.x, top_color.y, top_color.z
        ),
        World::Transparent => String::from(
            r#"
    # Setup transparent world
    world = bpy.context.scene.world
    if not world:
        world = bpy.data.worlds.new("World")
        bpy.context.scene.world = world
    
    world.use_nodes = True
    nodes = world.node_tree.nodes
    links = world.node_tree.links
    
    # Clear default nodes
    nodes.clear()
    
    # Create background node with black color
    background = nodes.new('ShaderNodeBackground')
    background.inputs['Color'].default_value = [0.0, 0.0, 0.0, 1.0]
    
    # Create output node
    output = nodes.new('ShaderNodeOutputWorld')
    
    # Link nodes
    links.new(background.outputs['Background'], output.inputs['Surface'])
    
    # Enable transparency
    bpy.context.scene.render.film_transparent = True
"#,
        ),
    }
}

fn generate_blender_script(scene: &Scene) -> String {
    let mut script = String::from(
        r#"import bpy
import mathutils
import sys

def setup_scene():
    # Clear existing scene
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()
    
    # Set up Cycles render engine
    bpy.context.scene.render.engine = 'CYCLES'
    bpy.context.scene.cycles.device = 'GPU'
    bpy.context.scene.cycles.samples = 128
    
    # Set units to meters
    bpy.context.scene.unit_settings.system = 'METRIC'
    bpy.context.scene.unit_settings.length_unit = 'METERS'
"#,
    );

    // Add world setup
    script.push_str(&generate_world_setup(&scene.world));

    // Add camera setup
    script.push_str(&generate_camera_setup(scene));

    // Add objects and materials
    for (i, object) in scene.objects.iter().enumerate() {
        script.push_str(&generate_material_setup(&object.material));
        script.push_str(&generate_object_setup(object, i));
    }

    // Add main execution with save
    script.push_str(
        r#"
    
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Please provide a .blend file path as argument")
        sys.exit(1)
        
    setup_scene()
    
    # Save the scene
    blend_file_path = sys.argv[-1]
    bpy.ops.wm.save_as_mainfile(filepath=blend_file_path)
"#,
    );

    script
}

fn main() -> color_eyre::Result<()> {
    use std::process::Command;

    let scene = Scene::default();
    let script = generate_blender_script(&scene);

    // Create temporary Python script
    let script_path = PathBuf::from("temp_scene.py");
    let mut file = File::create(&script_path)?;
    file.write_all(script.as_bytes())?;

    // Get output blend file path from command line or use default
    let blend_file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "scene.blend".to_string());

    // Run Blender with the Python script
    let status = Command::new("blender")
        .arg("--background") // Run without GUI
        .arg("--python")
        .arg(&script_path)
        .arg("--") // Separate Blender args from script args
        .arg(&blend_file_path)
        .status()?;

    // Clean up temporary script
    std::fs::remove_file(script_path)?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!("Blender execution failed"));
    }

    println!("Scene saved to: {}", blend_file_path);
    Ok(())
}
