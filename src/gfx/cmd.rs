use std::{marker::PhantomData, sync::Arc};

use vulkano::{
	DeviceSize,
	buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
	command_buffer::{
		AutoCommandBufferBuilder, CommandBufferExecFuture, CopyBufferToImageInfo,
		PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract, RenderingAttachmentInfo, RenderingInfo,
		SubpassContents,
	},
	device::Queue,
	format::Format,
	image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView},
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
	pipeline::graphics::vertex_input::Vertex,
	render_pass::{AttachmentLoadOp, AttachmentStoreOp},
	sync::{GpuFuture, future::NowFuture},
};

use super::{WGfx, pass::WGfxPass};

pub type GfxCommandBuffer = WCommandBuffer<CmdBufGfx>;
pub type XferCommandBuffer = WCommandBuffer<CmdBufXfer>;

pub struct CmdBufGfx;
pub struct CmdBufXfer;

pub struct WCommandBuffer<T> {
	pub(super) graphics: Arc<WGfx>,
	pub(super) queue: Arc<Queue>,
	pub(super) command_buffer: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
	pub(super) _dummy: PhantomData<T>,
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
