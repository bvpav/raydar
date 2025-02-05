use std::{iter, sync::Arc};

use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix};
use image::RgbaImage;
use shaders::raygen;
use vulkano::{
    acceleration_structure::{
        AabbPositions, AccelerationStructure, AccelerationStructureBuildGeometryInfo,
        AccelerationStructureBuildRangeInfo, AccelerationStructureBuildType,
        AccelerationStructureCreateInfo, AccelerationStructureGeometries,
        AccelerationStructureGeometryAabbsData, AccelerationStructureGeometryInstancesData,
        AccelerationStructureGeometryInstancesDataType, AccelerationStructureGeometryTrianglesData,
        AccelerationStructureInstance, AccelerationStructureType, BuildAccelerationStructureFlags,
        BuildAccelerationStructureMode,
    },
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, IndexBuffer, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyImageToBufferInfo, PrimaryCommandBufferAbstract,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        DescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures,
        Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    padded::Padded,
    pipeline::{
        graphics::vertex_input,
        layout::PipelineLayoutCreateInfo,
        ray_tracing::{
            RayTracingPipeline, RayTracingPipelineCreateInfo, RayTracingShaderGroupCreateInfo,
            ShaderBindingTable,
        },
        PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    shader::ShaderStages,
    sync::{self, GpuFuture},
    Packed24_8, Version, VulkanLibrary,
};

use crate::scene::{objects::Geometry, world::World, Scene};

use super::{timing::FrameTimer, Renderer, MAX_BOUNCES, MAX_SAMPLE_COUNT};

pub struct VulkanRenderer {
    timer: FrameTimer,

    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<RayTracingPipeline>,
    pipeline_layout: Arc<PipelineLayout>,
    shader_binding_table: ShaderBindingTable,

    cube_vertex_buffer: Subbuffer<[Vertex]>,
    cube_index_buffer: Subbuffer<[u32]>,
    cube_blas: Arc<AccelerationStructure>,
    sphere_blas: Arc<AccelerationStructure>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    bound_scene: Option<BoundScene>,
    sample_count: usize,
}

struct BoundScene {
    tlas: Arc<AccelerationStructure>,
    scene_descriptor_set: Arc<DescriptorSet>,
    image_descriptor_set: Arc<DescriptorSet>,
    image: Arc<Image>,
    image_view: Arc<ImageView>,
    output_buffer: Subbuffer<[u8]>,
}

#[derive(BufferContents, vertex_input::Vertex)]
#[repr(C)]
struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    normal: [f32; 3],
}

impl Renderer for VulkanRenderer {
    fn render_frame(&mut self, scene: &Scene) -> RgbaImage {
        self.new_frame(scene);

        let frame = self.render_sample(scene).unwrap();

        frame
    }

    fn render_sample(&mut self, scene: &Scene) -> Option<RgbaImage> {
        self.timer.start_sample();

        if self.sample_count >= MAX_SAMPLE_COUNT {
            return None;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let bound_scene = self.bound_scene.as_ref()?;

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::RayTracing,
                self.pipeline_layout.clone(),
                0,
                vec![
                    bound_scene.scene_descriptor_set.clone(),
                    bound_scene.image_descriptor_set.clone(),
                ],
            )
            .ok()?;

        builder
            .bind_pipeline_ray_tracing(self.pipeline.clone())
            .ok()?;

        let extent = bound_scene.image_view.image().extent();
        unsafe {
            builder
                .trace_rays(
                    self.shader_binding_table.addresses().clone(),
                    extent[0],
                    extent[1],
                    1,
                )
                .ok()?;
        }

        builder
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                bound_scene.image.clone(),
                bound_scene.output_buffer.clone(),
            ))
            .ok()?;

        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        // Read the buffer data and save it as an image
        let buffer_content = bound_scene.output_buffer.read().unwrap();

        self.sample_count = MAX_SAMPLE_COUNT;

        self.timer.end_sample();
        self.timer.end_frame();

        image::RgbaImage::from_raw(
            scene.camera.resolution_x(),
            scene.camera.resolution_y(),
            buffer_content.to_vec(),
        )
    }

    fn new_frame(&mut self, scene: &Scene) {
        let image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC,
                format: Format::R8G8B8A8_UNORM,
                extent: [scene.camera.resolution_x(), scene.camera.resolution_y(), 1],
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        let instances: Vec<_> = scene
            .objects
            .iter()
            .enumerate()
            // FIXME: Add support for spheres
            .map(|(index, object)| {
                let get_transform_matrix = |location: Point3<f32>, scale: f32| {
                    let transform =
                        Matrix4::from_translation(location.to_vec()) * Matrix4::from_scale(scale);
                    [
                        [transform.x.x, transform.x.y, transform.x.z, transform.w.x],
                        [transform.y.x, transform.y.y, transform.y.z, transform.w.y],
                        [transform.z.x, transform.z.y, transform.z.z, transform.w.z],
                    ]
                };

                match &object.geometry {
                    Geometry::Sphere(sphere) => AccelerationStructureInstance {
                        acceleration_structure_reference: self.sphere_blas.device_address().into(),
                        transform: get_transform_matrix(sphere.center, sphere.radius),
                        instance_custom_index_and_mask: Packed24_8::new(index as u32, 0xFF),
                        instance_shader_binding_table_record_offset_and_flags: Packed24_8::new(
                            1, 0,
                        ),
                        ..Default::default()
                    },
                    Geometry::Cube(cube) => AccelerationStructureInstance {
                        acceleration_structure_reference: self.cube_blas.device_address().into(),
                        transform: get_transform_matrix(cube.center, cube.side_length),
                        instance_custom_index_and_mask: Packed24_8::new(index as u32, 0xFF),
                        ..Default::default()
                    },
                }
            })
            .collect();

        let tlas = build_tlas(
            instances,
            self.device.clone(),
            self.memory_allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
        );

        // Convert from left-handed to right-handed coordinate system and flip Y
        let view = Matrix4::look_at_rh(
            scene.camera.position(),
            scene.camera.target(),
            scene.camera.up() * -1.0,
        );

        let camera_uniform_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            raygen::Camera {
                view_proj: (scene.camera.proj_matrix() * view).into(),
                inverse_view: view.invert().unwrap().into(),
                inverse_proj: scene.camera.inverse_proj_matrix().into(),
            },
        )
        .unwrap();

        let world_uniform_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            match scene.world {
                World::SkyColor {
                    top_color,
                    bottom_color,
                } => raygen::World {
                    top_color: Padded(top_color.into()),
                    bottom_color: bottom_color.into(),
                },
                World::SolidColor(color) => raygen::World {
                    top_color: Padded(color.into()),
                    bottom_color: color.into(),
                },
                World::Transparent => todo!(),
            },
        )
        .unwrap();

        let materials = scene.objects.iter().map(|object| raygen::Material {
            albedo: object.material.albedo.into(),
            roughness: object.material.roughness,
            metallic: Padded(object.material.metallic),
            emission_color: object.material.emission_color.into(),
            emission_strength: object.material.emission_strength,
            transmission: object.material.transmission,
            ior: object.material.ior,
            // HACK: This is done to fix an issue with the shader compiler and the data layout it generates
            _padding: [0.0, 0.0],
        });
        let materials_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            materials,
        )
        .unwrap();

        let sphere_data = scene.objects.iter().map(|object| match &object.geometry {
            Geometry::Sphere(sphere) => shaders::intersection::Sphere {
                center: sphere.center.into(),
                radius: sphere.radius,
            },
            _ => shaders::intersection::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 0.0,
            },
        });
        let sphere_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            sphere_data,
        )
        .unwrap();

        let renderer_properties = shaders::raygen::RendererProperties {
            max_bounces: MAX_BOUNCES as u32,
            max_sample_count: self.max_sample_count() as u32,
        };
        let renderer_properties_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            renderer_properties,
        )
        .unwrap();

        let scene_descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.pipeline_layout.set_layouts()[0].clone(),
            [
                WriteDescriptorSet::acceleration_structure(0, tlas.clone()),
                WriteDescriptorSet::buffer(1, camera_uniform_buffer),
                WriteDescriptorSet::buffer(2, world_uniform_buffer),
                WriteDescriptorSet::buffer(3, self.cube_vertex_buffer.clone()),
                WriteDescriptorSet::buffer(4, self.cube_index_buffer.clone()),
                WriteDescriptorSet::buffer(5, materials_buffer),
                WriteDescriptorSet::buffer(6, sphere_buffer),
                WriteDescriptorSet::buffer(7, renderer_properties_buffer),
            ],
            [],
        )
        .unwrap();

        let image_view = ImageView::new_default(image.clone()).unwrap();
        let image_descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.pipeline_layout.set_layouts()[1].clone(),
            [WriteDescriptorSet::image_view(0, image_view.clone())],
            [],
        )
        .unwrap();

        let scratch_memory_allocator =
            Arc::new(StandardMemoryAllocator::new_default(self.device.clone()));
        let output_buffer = Buffer::new_slice::<u8>(
            scratch_memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            (scene.camera.resolution_x() * scene.camera.resolution_y() * 4) as u64,
        )
        .unwrap();

        self.bound_scene = Some(BoundScene {
            tlas,
            scene_descriptor_set,
            image_descriptor_set,
            image_view,
            image,
            output_buffer,
        });
        self.timer.start_frame();
        self.sample_count = 0;
    }

    fn max_sample_count(&self) -> usize {
        MAX_SAMPLE_COUNT
    }

    fn timer(&self) -> &FrameTimer {
        &self.timer
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }
}

impl VulkanRenderer {
    pub fn new() -> Self {
        let library = VulkanLibrary::new().unwrap();
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: InstanceExtensions::empty(),
                ..Default::default()
            },
        )
        .unwrap();

        let device_extensions = DeviceExtensions {
            khr_ray_tracing_pipeline: true,
            khr_ray_tracing_maintenance1: true,
            khr_synchronization2: true,
            khr_deferred_host_operations: true,
            khr_acceleration_structure: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.api_version() >= Version::V1_3)
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(_, q)| {
                        q.queue_flags
                            .contains(QueueFlags::GRAPHICS | QueueFlags::COMPUTE)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_features: DeviceFeatures {
                    acceleration_structure: true,
                    ray_tracing_pipeline: true,
                    buffer_device_address: true,
                    synchronization2: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                set_layouts: vec![
                    DescriptorSetLayout::new(
                        device.clone(),
                        DescriptorSetLayoutCreateInfo {
                            bindings: [
                                // Acceleration structure binding
                                (
                                    0,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::RAYGEN,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::AccelerationStructure,
                                        )
                                    },
                                ),
                                // Camera uniform buffer binding
                                (
                                    1,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::RAYGEN,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::UniformBuffer,
                                        )
                                    },
                                ),
                                // World uniform buffer binding
                                (
                                    2,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::RAYGEN,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::UniformBuffer,
                                        )
                                    },
                                ),
                                // Vertex buffer binding
                                (
                                    3,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::CLOSEST_HIT,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::StorageBuffer,
                                        )
                                    },
                                ),
                                // Index buffer binding
                                (
                                    4,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::CLOSEST_HIT,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::StorageBuffer,
                                        )
                                    },
                                ),
                                // Materials buffer binding
                                (
                                    5,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::RAYGEN,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::StorageBuffer,
                                        )
                                    },
                                ),
                                // Sphere buffer binding
                                (
                                    6,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::INTERSECTION
                                            | ShaderStages::CLOSEST_HIT,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::StorageBuffer,
                                        )
                                    },
                                ),
                                // Renderer properties binding
                                (
                                    7,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::RAYGEN,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::UniformBuffer,
                                        )
                                    },
                                ),
                            ]
                            .into_iter()
                            .collect(),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                    DescriptorSetLayout::new(
                        device.clone(),
                        DescriptorSetLayoutCreateInfo {
                            bindings: [(
                                0,
                                DescriptorSetLayoutBinding {
                                    stages: ShaderStages::RAYGEN,
                                    ..DescriptorSetLayoutBinding::descriptor_type(
                                        DescriptorType::StorageImage,
                                    )
                                },
                            )]
                            .into_iter()
                            .collect(),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        let pipeline = {
            let raygen = shaders::raygen::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let triangle_closest_hit = shaders::triangle_closest_hit::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let sphere_closest_hit = shaders::sphere_closest_hit::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let miss = shaders::miss::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let intersection = shaders::intersection::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            // Make a list of the shader stages that the pipeline will have.
            let stages = [
                PipelineShaderStageCreateInfo::new(raygen),
                PipelineShaderStageCreateInfo::new(miss),
                PipelineShaderStageCreateInfo::new(triangle_closest_hit),
                PipelineShaderStageCreateInfo::new(sphere_closest_hit),
                PipelineShaderStageCreateInfo::new(intersection),
            ];

            // Define the shader groups that will eventually turn into the shader binding table.
            // The numbers are the indices of the stages in the `stages` array.
            let groups = [
                RayTracingShaderGroupCreateInfo::General { general_shader: 0 },
                RayTracingShaderGroupCreateInfo::General { general_shader: 1 },
                RayTracingShaderGroupCreateInfo::TrianglesHit {
                    closest_hit_shader: Some(2),
                    any_hit_shader: None,
                },
                RayTracingShaderGroupCreateInfo::ProceduralHit {
                    closest_hit_shader: Some(3),
                    any_hit_shader: None,
                    intersection_shader: 4,
                },
            ];

            RayTracingPipeline::new(
                device.clone(),
                None,
                RayTracingPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    groups: groups.into_iter().collect(),
                    max_pipeline_ray_recursion_depth: 1,

                    ..RayTracingPipelineCreateInfo::layout(pipeline_layout.clone())
                },
            )
            .unwrap()
        };

        let shader_binding_table =
            ShaderBindingTable::new(memory_allocator.clone(), &pipeline).unwrap();

        let (cube_vertex_buffer, cube_index_buffer) = {
            // TODO: Use Rust metaprogramming (macros) to generate the vertex data at compile time instead of hardcoding it manually
            let vertices = [
                // Left Top Front
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    normal: [0.0, 0.0, -1.0],
                },
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    normal: [-1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    normal: [0.0, 1.0, 0.0],
                },
                // Left Bottom Front
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    normal: [0.0, 0.0, -1.0],
                },
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    normal: [-1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    normal: [0.0, -1.0, 0.0],
                },
                // Right Top Front
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    normal: [0.0, 0.0, -1.0],
                },
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    normal: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    normal: [0.0, 1.0, 0.0],
                },
                // Right Bottom Front
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    normal: [0.0, 0.0, -1.0],
                },
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    normal: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    normal: [0.0, -1.0, 0.0],
                },
                // Left Top Back
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    normal: [0.0, -1.0, 0.0],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    normal: [0.0, 1.0, 0.0],
                },
                // Left Bottom Back
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    normal: [-1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    normal: [0.0, -1.0, 0.0],
                },
                // Right Top Back
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    normal: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    normal: [0.0, 1.0, 0.0],
                },
                // Right Bottom Back
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    normal: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    normal: [0.0, -1.0, 0.0],
                },
            ];
            let indices: [u32; 36] = [
                0, 3, 6, // Front 1
                6, 9, 3, // Front 2
                10, 7, 19, // Right 1
                19, 22, 10, // Right 2
                21, 18, 15, // Back 1
                15, 12, 18, // Back 2
                16, 13, 4, // Left 1
                4, 1, 13, // Left 2
                5, 11, 17, // Bottom 1
                17, 23, 11, // Bottom 2
                2, 8, 14, // Top 1
                14, 20, 8, // Top 2
            ];

            let vertex_buffer = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER
                        | BufferUsage::SHADER_DEVICE_ADDRESS
                        | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY
                        | BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vertices,
            )
            .unwrap();

            let index_buffer = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER
                        | BufferUsage::SHADER_DEVICE_ADDRESS
                        | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY
                        | BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                indices,
            )
            .unwrap();

            (vertex_buffer, index_buffer)
        };

        let cube_blas = {
            build_blas_triangles(
                cube_vertex_buffer.clone(),
                cube_index_buffer.clone(),
                device.clone(),
                memory_allocator.clone(),
                command_buffer_allocator.clone(),
                queue.clone(),
            )
        };

        let sphere_aabb_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::SHADER_DEVICE_ADDRESS
                    | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![AabbPositions {
                min: [-1.0, -1.0, -1.0],
                max: [1.0, 1.0, 1.0],
            }],
        )
        .unwrap();

        let sphere_blas = build_blas_aabb(
            sphere_aabb_buffer.clone(),
            device.clone(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            queue.clone(),
        );

        Self {
            timer: FrameTimer::default(),

            instance,
            device,
            queue,
            pipeline,
            pipeline_layout,
            shader_binding_table,

            cube_vertex_buffer,
            cube_index_buffer,
            cube_blas,
            sphere_blas,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,

            bound_scene: None,
            sample_count: 0,
        }
    }
}

fn build_blas_triangles(
    vertex_buffer: Subbuffer<[Vertex]>,
    index_buffer: Subbuffer<[u32]>,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Arc<AccelerationStructure> {
    let triangle_count = (index_buffer.len() / 3) as u32;
    let triangles_data = AccelerationStructureGeometryTrianglesData {
        max_vertex: vertex_buffer.len() as _,
        vertex_data: Some(vertex_buffer.into_bytes()),
        vertex_stride: size_of::<Vertex>() as _,
        index_data: Some(IndexBuffer::U32(index_buffer)),
        ..AccelerationStructureGeometryTrianglesData::new(Format::R32G32B32_SFLOAT)
    };
    let triangles_geometries = AccelerationStructureGeometries::Triangles(vec![triangles_data]);

    build_acceleration_structure(
        AccelerationStructureType::BottomLevel,
        triangles_geometries,
        triangle_count,
        device,
        memory_allocator,
        command_buffer_allocator,
        queue,
    )
}

fn build_blas_aabb(
    aabb_buffer: Subbuffer<[AabbPositions]>,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Arc<AccelerationStructure> {
    let aabb_data = AccelerationStructureGeometryAabbsData {
        data: Some(aabb_buffer.into_bytes()),
        stride: size_of::<AabbPositions>() as _,
        ..Default::default()
    };
    let aabb_geometries = AccelerationStructureGeometries::Aabbs(vec![aabb_data]);

    build_acceleration_structure(
        AccelerationStructureType::BottomLevel,
        aabb_geometries,
        1,
        device,
        memory_allocator,
        command_buffer_allocator,
        queue,
    )
}

fn build_tlas(
    instances: Vec<AccelerationStructureInstance>,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Arc<AccelerationStructure> {
    let instance_count = instances.len() as u32;
    let instance_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::SHADER_DEVICE_ADDRESS
                | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        instances,
    )
    .unwrap();

    let geometry_instances_data = AccelerationStructureGeometryInstancesData::new(
        AccelerationStructureGeometryInstancesDataType::Values(Some(instance_buffer)),
    );

    let geometries = AccelerationStructureGeometries::Instances(geometry_instances_data);

    build_acceleration_structure(
        AccelerationStructureType::TopLevel,
        geometries,
        instance_count,
        device,
        memory_allocator,
        command_buffer_allocator,
        queue,
    )
}

fn build_acceleration_structure(
    ty: AccelerationStructureType,
    geometries: AccelerationStructureGeometries,
    primitive_count: u32,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Arc<AccelerationStructure> {
    let mut build_geometry_info = AccelerationStructureBuildGeometryInfo {
        mode: BuildAccelerationStructureMode::Build,
        flags: BuildAccelerationStructureFlags::PREFER_FAST_TRACE,
        ..AccelerationStructureBuildGeometryInfo::new(geometries)
    };
    let build_sizes_info = device
        .acceleration_structure_build_sizes(
            AccelerationStructureBuildType::Device,
            &build_geometry_info,
            &[primitive_count],
        )
        .unwrap();

    let scratch_buffer = Buffer::new_slice::<u8>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::SHADER_DEVICE_ADDRESS | BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
        build_sizes_info.build_scratch_size,
    )
    .unwrap();

    let create_info = AccelerationStructureCreateInfo {
        ty,
        ..AccelerationStructureCreateInfo::new(
            Buffer::new_slice::<u8>(
                memory_allocator,
                BufferCreateInfo {
                    usage: BufferUsage::ACCELERATION_STRUCTURE_STORAGE
                        | BufferUsage::SHADER_DEVICE_ADDRESS,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
                build_sizes_info.acceleration_structure_size,
            )
            .unwrap(),
        )
    };

    let acceleration_structure =
        unsafe { AccelerationStructure::new(device, create_info).unwrap() };

    build_geometry_info.dst_acceleration_structure = Some(acceleration_structure.clone());
    build_geometry_info.scratch_data = Some(scratch_buffer);

    let range_info = AccelerationStructureBuildRangeInfo {
        primitive_count,
        ..Default::default()
    };

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    unsafe {
        builder
            .build_acceleration_structure(build_geometry_info, iter::once(range_info).collect())
            .unwrap();
    }

    builder
        .build()
        .unwrap()
        .execute(queue)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    acceleration_structure
}

mod shaders {
    pub(super) mod raygen {
        vulkano_shaders::shader! {
            ty: "raygen",
            path: "shaders/vulkan/raytrace.rgen",
            vulkan_version: "1.2"
        }
    }

    pub(super) mod triangle_closest_hit {
        vulkano_shaders::shader! {
            ty: "closesthit",
            path: "shaders/vulkan/raytrace_triangle.rchit",
            vulkan_version: "1.2"
        }
    }

    pub(super) mod sphere_closest_hit {
        vulkano_shaders::shader! {
            ty: "closesthit",
            path: "shaders/vulkan/raytrace_sphere.rchit",
            vulkan_version: "1.2"
        }
    }

    pub(super) mod miss {
        vulkano_shaders::shader! {
            ty: "miss",
            path: "shaders/vulkan/raytrace.miss",
            vulkan_version: "1.2"
        }
    }

    pub(super) mod intersection {
        vulkano_shaders::shader! {
            ty: "intersection",
            path: "shaders/vulkan/raytrace.rint",
            vulkan_version: "1.2"
        }
    }
}
