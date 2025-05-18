use std::{marker::PhantomData, ops::Range, sync::Arc};

use smallvec::smallvec;
use vulkano::{
	buffer::{
		BufferContents, BufferUsage, IndexBuffer, Subbuffer,
		allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
	},
	descriptor_set::{DescriptorSet, WriteDescriptorSet},
	format::Format,
	image::{
		sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
		view::ImageView,
	},
	memory::allocator::MemoryTypeFilter,
	pipeline::{
		DynamicState, GraphicsPipeline, Pipeline, PipelineLayout,
		graphics::{
			GraphicsPipelineCreateInfo,
			color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
			input_assembly::{InputAssemblyState, PrimitiveTopology},
			multisample::MultisampleState,
			rasterization::RasterizationState,
			subpass::PipelineRenderingCreateInfo,
			vertex_input::{Vertex, VertexDefinition},
			viewport::ViewportState,
		},
		layout::PipelineDescriptorSetLayoutCreateInfo,
	},
	shader::ShaderModule,
};

use super::{WGfx, pass::WGfxPass};

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
	pub(super) fn new(
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

	pub fn create_pass_instanced(
		self: &Arc<Self>,
		dimensions: [f32; 2],
		vertex_buffer: Subbuffer<[V]>,
		vertices: Range<u32>,
		instances: Range<u32>,
		descriptor_sets: Vec<Arc<DescriptorSet>>,
	) -> anyhow::Result<WGfxPass<V>> {
		WGfxPass::new_instanced(
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
