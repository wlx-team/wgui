use std::{marker::PhantomData, ops::Range, slice::Iter, sync::Arc};

use smallvec::smallvec;
use vulkano::{
	DeviceSize,
	buffer::{
		Buffer, BufferContents, BufferCreateInfo, BufferUsage, IndexBuffer, Subbuffer,
		allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
	},
	command_buffer::{
		AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferInheritanceInfo,
		CommandBufferInheritanceRenderPassType, CommandBufferInheritanceRenderingInfo,
		CommandBufferUsage, CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
		PrimaryCommandBufferAbstract, RenderingAttachmentInfo, RenderingInfo,
		SecondaryAutoCommandBuffer, SubpassContents,
		allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
	},
	descriptor_set::{
		DescriptorSet, WriteDescriptorSet,
		allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo},
	},
	device::{Device, Queue},
	format::Format,
	image::{
		Image, ImageCreateInfo, ImageType, ImageUsage,
		sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
		view::ImageView,
	},
	instance::Instance,
	memory::{
		MemoryPropertyFlags,
		allocator::{
			AllocationCreateInfo, GenericMemoryAllocatorCreateInfo, MemoryTypeFilter,
			StandardMemoryAllocator,
		},
	},
	pipeline::{
		DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
		graphics::{
			GraphicsPipelineCreateInfo,
			color_blend::{
				AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
			},
			input_assembly::{InputAssemblyState, PrimitiveTopology},
			multisample::MultisampleState,
			rasterization::RasterizationState,
			subpass::PipelineRenderingCreateInfo,
			vertex_input::{Vertex, VertexDefinition},
			viewport::{Viewport, ViewportState},
		},
		layout::PipelineDescriptorSetLayoutCreateInfo,
	},
	render_pass::{AttachmentLoadOp, AttachmentStoreOp},
	shader::ShaderModule,
	sync::{GpuFuture, future::NowFuture},
};

pub const BLEND_ALPHA: AttachmentBlend = AttachmentBlend {
	src_color_blend_factor: BlendFactor::SrcAlpha,
	dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
	color_blend_op: BlendOp::Add,
	src_alpha_blend_factor: BlendFactor::One,
	dst_alpha_blend_factor: BlendFactor::One,
	alpha_blend_op: BlendOp::Max,
};

pub type Vert2Buf = Subbuffer<[Vert2Uv]>;
pub type IndexBuf = IndexBuffer;
#[repr(C)]
#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
pub struct Vert2Uv {
	#[format(R32G32_SFLOAT)]
	pub in_pos: [f32; 2],
	#[format(R32G32_SFLOAT)]
	pub in_uv: [f32; 2],
}

pub const INDICES: [u16; 6] = [2, 1, 0, 1, 2, 3];
#[derive(Clone)]
pub struct WGfx {
	pub instance: Arc<Instance>,
	pub device: Arc<Device>,

	pub queue_gfx: Arc<Queue>,
	pub queue_xfer: Arc<Queue>,

	pub memory_allocator: Arc<StandardMemoryAllocator>,
	pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
	pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl WGfx {
	pub fn new_from_raw(
		instance: Arc<Instance>,
		device: Arc<Device>,
		queue_gfx: Arc<Queue>,
		queue_xfer: Arc<Queue>,
	) -> Arc<Self> {
		let memory_allocator = memory_allocator(device.clone());
		let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
			device.clone(),
			StandardCommandBufferAllocatorCreateInfo {
				secondary_buffer_count: 32,
				..Default::default()
			},
		));
		let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
			device.clone(),
			StandardDescriptorSetAllocatorCreateInfo::default(),
		));

		Arc::new(Self {
			instance,
			device,
			queue_gfx,
			queue_xfer,
			memory_allocator,
			command_buffer_allocator,
			descriptor_set_allocator,
		})
	}
	pub fn upload_verts(
		&self,
		width: f32,
		height: f32,
		x: f32,
		y: f32,
		w: f32,
		h: f32,
	) -> anyhow::Result<Vert2Buf> {
		let rw = width;
		let rh = height;

		let x0 = x / rw;
		let y0 = y / rh;

		let x1 = w / rw + x0;
		let y1 = h / rh + y0;

		let vertices = [
			Vert2Uv {
				in_pos: [x0, y0],
				in_uv: [0.0, 0.0],
			},
			Vert2Uv {
				in_pos: [x0, y1],
				in_uv: [0.0, 1.0],
			},
			Vert2Uv {
				in_pos: [x1, y0],
				in_uv: [1.0, 0.0],
			},
			Vert2Uv {
				in_pos: [x1, y1],
				in_uv: [1.0, 1.0],
			},
		];
		self.new_buffer(BufferUsage::VERTEX_BUFFER, vertices.iter())
	}

	pub fn empty_buffer<T>(&self, usage: BufferUsage, capacity: u64) -> anyhow::Result<Subbuffer<[T]>>
	where
		T: BufferContents + Clone,
	{
		Ok(Buffer::new_slice(
			self.memory_allocator.clone(),
			BufferCreateInfo {
				usage,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			capacity,
		)?)
	}

	pub fn new_buffer<T>(
		&self,
		usage: BufferUsage,
		contents: Iter<'_, T>,
	) -> anyhow::Result<Subbuffer<[T]>>
	where
		T: BufferContents + Clone,
	{
		Ok(Buffer::from_iter(
			self.memory_allocator.clone(),
			BufferCreateInfo {
				usage,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			contents.cloned(),
		)?)
	}

	pub fn new_image(
		&self,
		width: u32,
		height: u32,
		format: Format,
		usage: ImageUsage,
	) -> anyhow::Result<Arc<Image>> {
		Ok(Image::new(
			self.memory_allocator.clone(),
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format,
				extent: [width, height, 1],
				usage,
				..Default::default()
			},
			AllocationCreateInfo::default(),
		)?)
	}

	pub fn create_pipeline<V>(
		self: &Arc<Self>,
		vert: Arc<ShaderModule>,
		frag: Arc<ShaderModule>,
		format: Format,
		blend: Option<AttachmentBlend>,
		topology: PrimitiveTopology,
		instanced: bool,
	) -> anyhow::Result<Arc<WGfxPipeline<V>>>
	where
		V: BufferContents + Vertex,
	{
		Ok(Arc::new(WGfxPipeline::new(
			self.clone(),
			vert,
			frag,
			format,
			blend,
			topology,
			instanced,
		)?))
	}

	pub fn create_gfx_command_buffer(
		self: &Arc<Self>,
		usage: CommandBufferUsage,
	) -> anyhow::Result<GfxCommandBuffer> {
		let command_buffer = AutoCommandBufferBuilder::primary(
			self.command_buffer_allocator.clone(),
			self.queue_gfx.queue_family_index(),
			usage,
		)?;
		Ok(GfxCommandBuffer {
			graphics: self.clone(),
			queue: self.queue_gfx.clone(),
			command_buffer,
			_dummy: PhantomData,
		})
	}

	pub fn create_xfer_command_buffer(
		self: &Arc<Self>,
		usage: CommandBufferUsage,
	) -> anyhow::Result<XferCommandBuffer> {
		let command_buffer = AutoCommandBufferBuilder::primary(
			self.command_buffer_allocator.clone(),
			self.queue_xfer.queue_family_index(),
			usage,
		)?;
		Ok(XferCommandBuffer {
			graphics: self.clone(),
			queue: self.queue_xfer.clone(),
			command_buffer,
			_dummy: PhantomData,
		})
	}
}

pub type GfxCommandBuffer = WCommandBuffer<CmdBufGfx>;
pub type XferCommandBuffer = WCommandBuffer<CmdBufXfer>;

pub struct CmdBufGfx;
pub struct CmdBufXfer;

pub struct WCommandBuffer<T> {
	pub graphics: Arc<WGfx>,
	pub command_buffer: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
	pub queue: Arc<Queue>,
	_dummy: PhantomData<T>,
}

impl<T> WCommandBuffer<T> {
	pub fn build_and_execute(self) -> anyhow::Result<CommandBufferExecFuture<NowFuture>> {
		let queue = self.queue.clone();
		Ok(self.command_buffer.build()?.execute(queue)?)
	}

	pub fn build_and_execute_now(self) -> anyhow::Result<()> {
		let mut exec = self.build_and_execute()?;
		exec.flush()?;
		exec.cleanup_finished();
		Ok(())
	}
}

impl WCommandBuffer<CmdBufGfx> {
	pub fn begin_rendering(&mut self, render_target: Arc<ImageView>) -> anyhow::Result<()> {
		self.command_buffer.begin_rendering(RenderingInfo {
			contents: SubpassContents::SecondaryCommandBuffers,
			color_attachments: vec![Some(RenderingAttachmentInfo {
				load_op: AttachmentLoadOp::Clear,
				store_op: AttachmentStoreOp::Store,
				clear_value: Some([0.0, 0.0, 0.0, 0.0].into()),
				..RenderingAttachmentInfo::image_view(render_target)
			})],
			..Default::default()
		})?;
		Ok(())
	}

	pub fn build(self) -> anyhow::Result<Arc<PrimaryAutoCommandBuffer>> {
		Ok(self.command_buffer.build()?)
	}

	pub fn run_ref<T>(&mut self, pass: &WGfxPass<T>) -> anyhow::Result<()>
	where
		T: BufferContents + Vertex,
	{
		self
			.command_buffer
			.execute_commands(pass.command_buffer.clone())?;
		Ok(())
	}

	pub fn end_rendering(&mut self) -> anyhow::Result<()> {
		self.command_buffer.end_rendering()?;
		Ok(())
	}
}

impl WCommandBuffer<CmdBufXfer> {
	pub fn upload_image(
		&mut self,
		width: u32,
		height: u32,
		format: Format,
		data: &[u8],
	) -> anyhow::Result<Arc<Image>> {
		let image = Image::new(
			self.graphics.memory_allocator.clone(),
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format,
				extent: [width, height, 1],
				usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::SAMPLED,
				..Default::default()
			},
			AllocationCreateInfo::default(),
		)?;

		let buffer: Subbuffer<[u8]> = Buffer::new_slice(
			self.graphics.memory_allocator.clone(),
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_SRC,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			data.len() as DeviceSize,
		)?;

		buffer.write()?.copy_from_slice(data);

		self
			.command_buffer
			.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone()))?;

		Ok(image)
	}

	pub fn update_image(
		&mut self,
		image: Arc<Image>,
		data: &[u8],
		offset: [u32; 3],
		extent: Option<[u32; 3]>,
	) -> anyhow::Result<()> {
		let buffer: Subbuffer<[u8]> = Buffer::new_slice(
			self.graphics.memory_allocator.clone(),
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_SRC,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			data.len() as DeviceSize,
		)?;

		buffer.write()?.copy_from_slice(data);

		let mut copy_info = CopyBufferToImageInfo::buffer_image(buffer, image.clone());
		copy_info.regions[0].image_offset = offset;
		if let Some(extent) = extent {
			copy_info.regions[0].image_extent = extent;
		}

		self.command_buffer.copy_buffer_to_image(copy_info)?;
		Ok(())
	}
}

pub struct WGfxPipeline<V>
where
	V: BufferContents + Vertex,
{
	pub graphics: Arc<WGfx>,
	pub pipeline: Arc<GraphicsPipeline>,
	pub format: Format,
	_dummy: PhantomData<V>,
}

impl<V> WGfxPipeline<V>
where
	V: BufferContents + Vertex,
{
	fn new(
		graphics: Arc<WGfx>,
		vert: Arc<ShaderModule>,
		frag: Arc<ShaderModule>,
		format: Format,
		blend: Option<AttachmentBlend>,
		topology: PrimitiveTopology,
		instanced: bool,
	) -> anyhow::Result<Self> {
		let vep = vert.entry_point("main").unwrap(); // want panic
		let fep = frag.entry_point("main").unwrap(); // want panic

		let vertex_input_state = if instanced {
			V::per_instance().definition(&vep)?
		} else {
			V::per_vertex().definition(&vep)?
		};

		let stages = smallvec![
			vulkano::pipeline::PipelineShaderStageCreateInfo::new(vep),
			vulkano::pipeline::PipelineShaderStageCreateInfo::new(fep),
		];

		let layout = PipelineLayout::new(
			graphics.device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
				.into_pipeline_layout_create_info(graphics.device.clone())?,
		)?;

		let subpass = PipelineRenderingCreateInfo {
			color_attachment_formats: vec![Some(format)],
			..Default::default()
		};

		let pipeline = GraphicsPipeline::new(
			graphics.device.clone(),
			None,
			GraphicsPipelineCreateInfo {
				stages,
				vertex_input_state: Some(vertex_input_state),
				input_assembly_state: Some(InputAssemblyState {
					topology,
					..InputAssemblyState::default()
				}),
				viewport_state: Some(ViewportState::default()),
				rasterization_state: Some(RasterizationState {
					cull_mode: vulkano::pipeline::graphics::rasterization::CullMode::None,
					..RasterizationState::default()
				}),
				multisample_state: Some(MultisampleState::default()),
				color_blend_state: Some(ColorBlendState {
					attachments: vec![ColorBlendAttachmentState {
						blend,
						..Default::default()
					}],
					..Default::default()
				}),
				dynamic_state: std::iter::once(DynamicState::Viewport).collect(),
				subpass: Some(subpass.into()),
				..GraphicsPipelineCreateInfo::layout(layout)
			},
		)?;

		Ok(Self {
			graphics,
			pipeline,
			format,
			_dummy: PhantomData,
		})
	}

	pub fn create_pass_vertices(
		self: &Arc<Self>,
		dimensions: [f32; 2],
		vertex_buffer: Subbuffer<[V]>,
		vertices: Range<u32>,
		instances: Range<u32>,
		descriptor_sets: Vec<Arc<DescriptorSet>>,
	) -> anyhow::Result<WGfxPass<V>> {
		WGfxPass::new_vertices(
			self.clone(),
			dimensions,
			vertex_buffer,
			vertices,
			instances,
			descriptor_sets,
		)
	}

	pub fn create_pass_indexed(
		self: &Arc<Self>,
		dimensions: [f32; 2],
		vertex_buffer: Subbuffer<[V]>,
		index_buffer: IndexBuffer,
		descriptor_sets: Vec<Arc<DescriptorSet>>,
	) -> anyhow::Result<WGfxPass<V>> {
		WGfxPass::new_indexed(
			self.clone(),
			dimensions,
			vertex_buffer,
			index_buffer,
			descriptor_sets,
		)
	}

	pub fn inner(&self) -> Arc<GraphicsPipeline> {
		self.pipeline.clone()
	}

	pub fn uniform_sampler(
		&self,
		set: usize,
		texture: Arc<ImageView>,
		filter: Filter,
	) -> anyhow::Result<Arc<DescriptorSet>> {
		let sampler = Sampler::new(
			self.graphics.device.clone(),
			SamplerCreateInfo {
				mag_filter: filter,
				min_filter: filter,
				address_mode: [SamplerAddressMode::Repeat; 3],
				..Default::default()
			},
		)?;

		let layout = self.pipeline.layout().set_layouts().get(set).unwrap(); // want panic

		Ok(DescriptorSet::new(
			self.graphics.descriptor_set_allocator.clone(),
			layout.clone(),
			[WriteDescriptorSet::image_view_sampler(0, texture, sampler)],
			[],
		)?)
	}

	pub fn uniform_buffer<T>(
		&self,
		set: usize,
		buffer: Subbuffer<[T]>,
	) -> anyhow::Result<Arc<DescriptorSet>>
	where
		T: BufferContents + Copy,
	{
		let layout = self.pipeline.layout().set_layouts().get(set).unwrap(); // want panic
		Ok(DescriptorSet::new(
			self.graphics.descriptor_set_allocator.clone(),
			layout.clone(),
			[WriteDescriptorSet::buffer(0, buffer)],
			[],
		)?)
	}

	pub fn uniform_buffer_upload<T>(
		&self,
		set: usize,
		data: Vec<T>,
	) -> anyhow::Result<Arc<DescriptorSet>>
	where
		T: BufferContents + Copy,
	{
		let buf = SubbufferAllocator::new(
			self.graphics.memory_allocator.clone(),
			SubbufferAllocatorCreateInfo {
				buffer_usage: BufferUsage::UNIFORM_BUFFER,
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
					| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
		);

		let uniform_buffer_subbuffer = {
			let subbuffer = buf.allocate_slice(data.len() as _)?;
			subbuffer.write()?.copy_from_slice(data.as_slice());
			subbuffer
		};

		self.uniform_buffer(set, uniform_buffer_subbuffer)
	}
}

pub struct WGfxPass<V>
where
	V: BufferContents + Vertex,
{
	pub command_buffer: Arc<SecondaryAutoCommandBuffer>,
	_dummy: PhantomData<V>,
}

impl<V> WGfxPass<V>
where
	V: BufferContents + Vertex,
{
	fn new_indexed(
		pipeline: Arc<WGfxPipeline<V>>,
		dimensions: [f32; 2],
		vertex_buffer: Subbuffer<[V]>,
		index_buffer: IndexBuffer,
		descriptor_sets: Vec<Arc<DescriptorSet>>,
	) -> anyhow::Result<Self> {
		let viewport = Viewport {
			offset: [0.0, 0.0],
			extent: dimensions,
			depth_range: 0.0..=1.0,
		};
		let pipeline_inner = pipeline.inner();
		let mut command_buffer = AutoCommandBufferBuilder::secondary(
			pipeline.graphics.command_buffer_allocator.clone(),
			pipeline.graphics.queue_gfx.queue_family_index(),
			CommandBufferUsage::MultipleSubmit,
			CommandBufferInheritanceInfo {
				render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRendering(
					CommandBufferInheritanceRenderingInfo {
						color_attachment_formats: vec![Some(pipeline.format)],

						..Default::default()
					},
				)),
				..Default::default()
			},
		)?;

		unsafe {
			command_buffer
				.set_viewport(0, smallvec![viewport])?
				.bind_pipeline_graphics(pipeline_inner)?
				.bind_descriptor_sets(
					PipelineBindPoint::Graphics,
					pipeline.inner().layout().clone(),
					0,
					descriptor_sets,
				)?
				.bind_vertex_buffers(0, vertex_buffer)?
				.bind_index_buffer(index_buffer.clone())?
				.draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)?
		};

		Ok(Self {
			command_buffer: command_buffer.build()?,
			_dummy: PhantomData,
		})
	}

	fn new_vertices(
		pipeline: Arc<WGfxPipeline<V>>,
		dimensions: [f32; 2],
		vertex_buffer: Subbuffer<[V]>,
		vertices: Range<u32>,
		instances: Range<u32>,
		descriptor_sets: Vec<Arc<DescriptorSet>>,
	) -> anyhow::Result<Self> {
		let viewport = Viewport {
			offset: [0.0, 0.0],
			extent: dimensions,
			depth_range: 0.0..=1.0,
		};
		let pipeline_inner = pipeline.inner();
		let mut command_buffer = AutoCommandBufferBuilder::secondary(
			pipeline.graphics.command_buffer_allocator.clone(),
			pipeline.graphics.queue_gfx.queue_family_index(),
			CommandBufferUsage::MultipleSubmit,
			CommandBufferInheritanceInfo {
				render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRendering(
					CommandBufferInheritanceRenderingInfo {
						color_attachment_formats: vec![Some(pipeline.format)],

						..Default::default()
					},
				)),
				..Default::default()
			},
		)?;

		unsafe {
			command_buffer
				.set_viewport(0, smallvec![viewport])?
				.bind_pipeline_graphics(pipeline_inner)?
				.bind_descriptor_sets(
					PipelineBindPoint::Graphics,
					pipeline.inner().layout().clone(),
					0,
					descriptor_sets,
				)?
				.bind_vertex_buffers(0, vertex_buffer)?
				.draw(
					vertices.end - vertices.start, //TODO: verify
					instances.end - instances.start,
					vertices.start,
					instances.start,
				)?
		};

		Ok(Self {
			command_buffer: command_buffer.build()?,
			_dummy: PhantomData,
		})
	}
}

fn memory_allocator(device: Arc<Device>) -> Arc<StandardMemoryAllocator> {
	let props = device.physical_device().memory_properties();

	let mut block_sizes = vec![0; props.memory_types.len()];
	let mut memory_type_bits = u32::MAX;

	for (index, memory_type) in props.memory_types.iter().enumerate() {
		const LARGE_HEAP_THRESHOLD: DeviceSize = 1024 * 1024 * 1024;

		let heap_size = props.memory_heaps[memory_type.heap_index as usize].size;

		block_sizes[index] = if heap_size >= LARGE_HEAP_THRESHOLD {
			48 * 1024 * 1024
		} else {
			24 * 1024 * 1024
		};

		if memory_type.property_flags.intersects(
			MemoryPropertyFlags::LAZILY_ALLOCATED
				| MemoryPropertyFlags::PROTECTED
				| MemoryPropertyFlags::DEVICE_COHERENT
				| MemoryPropertyFlags::RDMA_CAPABLE,
		) {
			// VUID-VkMemoryAllocateInfo-memoryTypeIndex-01872
			// VUID-vkAllocateMemory-deviceCoherentMemory-02790
			// Lazily allocated memory would just cause problems for suballocation in general.
			memory_type_bits &= !(1 << index);
		}
	}

	let create_info = GenericMemoryAllocatorCreateInfo {
		block_sizes: &block_sizes,
		memory_type_bits,
		..Default::default()
	};

	Arc::new(StandardMemoryAllocator::new(device, create_info))
}
