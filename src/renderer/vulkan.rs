use std::{iter, sync::Arc};

use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use image::RgbaImage;
use shaders::raygen;
use vulkano::{
    acceleration_structure::{
        AccelerationStructure, AccelerationStructureBuildGeometryInfo,
        AccelerationStructureBuildRangeInfo, AccelerationStructureBuildType,
        AccelerationStructureCreateInfo, AccelerationStructureGeometries,
        AccelerationStructureGeometryInstancesData, AccelerationStructureGeometryInstancesDataType,
        AccelerationStructureGeometryTrianglesData, AccelerationStructureInstance,
        AccelerationStructureType, BuildAccelerationStructureFlags, BuildAccelerationStructureMode,
    },
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
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
    Version, VulkanLibrary,
};

use crate::scene::Scene;

use super::{timing::FrameTimer, Renderer, MAX_SAMPLE_COUNT};

pub struct VulkanRenderer {
    timer: FrameTimer,

    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<RayTracingPipeline>,
    pipeline_layout: Arc<PipelineLayout>,
    shader_binding_table: ShaderBindingTable,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    bound_scene: Option<BoundScene>,
    sample_count: usize,
}

struct BoundScene {
    tlas: Arc<AccelerationStructure>,
    blas: Arc<AccelerationStructure>,
    scene_descriptor_set: Arc<DescriptorSet>,
    image_descriptor_set: Arc<DescriptorSet>,
    image: Arc<Image>,
    image_view: Arc<ImageView>,
}

#[derive(BufferContents, vertex_input::Vertex)]
#[repr(C)]
struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
}

impl Renderer for VulkanRenderer {
    fn render_frame(&mut self, scene: &Scene) -> RgbaImage {
        self.new_frame(scene);

        let frame = self.render_sample(scene).unwrap();

        self.timer.end_frame();
        frame
    }

    fn render_sample(&mut self, scene: &Scene) -> Option<RgbaImage> {
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

        let output_buffer = Buffer::new_slice::<u8>(
            self.memory_allocator.clone(),
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
        .ok()?;

        builder
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                bound_scene.image.clone(),
                output_buffer.clone(),
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
        let buffer_content = output_buffer.read().ok()?;

        self.sample_count = MAX_SAMPLE_COUNT;

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

        let vertices = [
            Vertex {
                position: [-0.5, -0.25, 0.0],
            },
            Vertex {
                position: [0.0, 0.5, 0.0],
            },
            Vertex {
                position: [0.25, -0.1, 0.0],
            },
        ];
        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER
                    | BufferUsage::SHADER_DEVICE_ADDRESS
                    | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY,
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

        let blas = build_blas_triangles(
            vertex_buffer,
            self.device.clone(),
            self.memory_allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
        );

        let tlas = build_tlas(
            blas.clone(),
            self.device.clone(),
            self.memory_allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
        );

        // Convert from left-handed to right-handed coordinate system and flip Y
        let view = {
            let mut right_handed = scene.camera.view_matrix();
            // Flip the Z coordinates by negating the third row and column
            right_handed.z.x *= -1.0;
            right_handed.z.y *= -1.0;
            right_handed.z.z *= -1.0;
            right_handed.x.z *= -1.0;
            right_handed.y.z *= -1.0;
            right_handed.w.z *= -1.0;
            // Flip Y by negating the Y scale
            right_handed.y.y *= -1.0;
            right_handed
        };

        let uniform_buffer = Buffer::from_data(
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

        let scene_descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.pipeline_layout.set_layouts()[0].clone(),
            [
                WriteDescriptorSet::acceleration_structure(0, tlas.clone()),
                WriteDescriptorSet::buffer(1, uniform_buffer.clone()),
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

        self.bound_scene = Some(BoundScene {
            tlas,
            blas,
            scene_descriptor_set,
            image_descriptor_set,
            image_view,
            image,
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
                                (
                                    0,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::RAYGEN,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::AccelerationStructure,
                                        )
                                    },
                                ),
                                (
                                    1,
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
            let closest_hit = shaders::closest_hit::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let miss = shaders::miss::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            // Make a list of the shader stages that the pipeline will have.
            let stages = [
                PipelineShaderStageCreateInfo::new(raygen),
                PipelineShaderStageCreateInfo::new(miss),
                PipelineShaderStageCreateInfo::new(closest_hit),
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

        Self {
            timer: FrameTimer::default(),

            instance,
            device,
            queue,
            pipeline,
            pipeline_layout,
            shader_binding_table,

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
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Arc<AccelerationStructure> {
    let triangle_count = (vertex_buffer.len() / 3) as u32;
    let triangles_data = AccelerationStructureGeometryTrianglesData {
        max_vertex: vertex_buffer.len() as _,
        vertex_data: Some(vertex_buffer.into_bytes()),
        vertex_stride: size_of::<Vertex>() as _,
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

fn build_tlas(
    blas: Arc<AccelerationStructure>,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Arc<AccelerationStructure> {
    let instance = AccelerationStructureInstance {
        acceleration_structure_reference: blas.device_address().into(),
        ..Default::default()
    };

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
        [instance],
    )
    .unwrap();

    let geometry_instances_data = AccelerationStructureGeometryInstancesData::new(
        AccelerationStructureGeometryInstancesDataType::Values(Some(instance_buffer)),
    );

    let geometries = AccelerationStructureGeometries::Instances(geometry_instances_data);

    build_acceleration_structure(
        AccelerationStructureType::TopLevel,
        geometries,
        1,
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

    pub(super) mod closest_hit {
        vulkano_shaders::shader! {
            ty: "closesthit",
            path: "shaders/vulkan/raytrace.rchit",
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
}
