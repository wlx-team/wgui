use std::sync::{Arc, OnceLock};
use wgui::gfx::WGfx;
use wgui::vulkano::{
	self, DeviceSize,
	command_buffer::allocator::{
		StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
	},
	descriptor_set::allocator::StandardDescriptorSetAllocator,
	device::{
		Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags,
		physical::{PhysicalDevice, PhysicalDeviceType},
	},
	instance::{Instance, InstanceCreateInfo},
	memory::{
		MemoryPropertyFlags,
		allocator::{GenericMemoryAllocatorCreateInfo, StandardMemoryAllocator},
	},
};

static VULKAN_LIBRARY: OnceLock<Arc<vulkano::VulkanLibrary>> = OnceLock::new();
fn get_vulkan_library() -> &'static Arc<vulkano::VulkanLibrary> {
	VULKAN_LIBRARY.get_or_init(|| vulkano::VulkanLibrary::new().unwrap()) // want panic
}

pub fn init_window() -> anyhow::Result<(
	Arc<WGfx>,
	winit::event_loop::EventLoop<()>,
	Arc<winit::window::Window>,
	Arc<vulkano::swapchain::Surface>,
)> {
	use vulkano::{
		descriptor_set::allocator::StandardDescriptorSetAllocatorCreateInfo,
		instance::InstanceCreateFlags, swapchain::Surface,
	};
	use winit::{event_loop::EventLoop, window::Window};

	let event_loop = EventLoop::new().unwrap(); // want panic
	let mut vk_instance_extensions = Surface::required_extensions(&event_loop).unwrap();
	vk_instance_extensions.khr_get_physical_device_properties2 = true;
	log::debug!("Instance exts for runtime: {:?}", &vk_instance_extensions);

	let instance = Instance::new(
		get_vulkan_library().clone(),
		InstanceCreateInfo {
			flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
			enabled_extensions: vk_instance_extensions,
			..Default::default()
		},
	)?;

	#[allow(deprecated)]
	let window = Arc::new(
		event_loop
			.create_window(Window::default_attributes())
			.unwrap(), // want panic
	);
	let surface = Surface::from_window(instance.clone(), window.clone())?;

	let mut device_extensions = DeviceExtensions::empty();
	device_extensions.khr_swapchain = true;

	log::debug!("Device exts for app: {:?}", &device_extensions);

	let (physical_device, my_extensions, queue_families) = instance
		.enumerate_physical_devices()?
		.filter_map(|p| {
			if p.supported_extensions().contains(&device_extensions) {
				Some((p, device_extensions))
			} else {
				log::debug!(
					"Not using {} because it does not implement the following device extensions:",
					p.properties().device_name,
				);
				for (ext, missing) in p.supported_extensions().difference(&device_extensions) {
					if missing {
						log::debug!("  {ext}");
					}
				}
				None
			}
		})
		.filter_map(|(p, my_extensions)| {
			try_all_queue_families(p.as_ref()).map(|families| (p, my_extensions, families))
		})
		.min_by_key(|(p, _, _)| prio_from_device_type(p))
		.expect("no suitable physical device found");

	log::info!(
		"Using vkPhysicalDevice: {}",
		physical_device.properties().device_name,
	);

	let (device, queues) = Device::new(
		physical_device,
		DeviceCreateInfo {
			enabled_extensions: my_extensions,
			enabled_features: DeviceFeatures {
				dynamic_rendering: true,
				..DeviceFeatures::empty()
			},
			queue_create_infos: queue_families
				.iter()
				.map(|fam| QueueCreateInfo {
					queue_family_index: fam.queue_family_index,
					queues: fam.priorities.clone(),
					..Default::default()
				})
				.collect::<Vec<_>>(),
			..Default::default()
		},
	)?;

	let (queue_gfx, queue_xfer, _) = unwrap_queues(queues.collect());

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

	let me = WGfx {
		instance,
		device,
		queue_gfx,
		queue_xfer,
		memory_allocator,
		command_buffer_allocator,
		descriptor_set_allocator,
	};

	Ok((Arc::new(me), event_loop, window, surface))
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

#[derive(Debug)]
struct QueueFamilyLayout {
	queue_family_index: u32,
	priorities: Vec<f32>,
}

fn prio_from_device_type(physical_device: &PhysicalDevice) -> u32 {
	match physical_device.properties().device_type {
		PhysicalDeviceType::DiscreteGpu => 0,
		PhysicalDeviceType::IntegratedGpu => 1,
		PhysicalDeviceType::VirtualGpu => 2,
		PhysicalDeviceType::Cpu => 3,
		_ => 4,
	}
}

fn unwrap_queues(queues: Vec<Arc<Queue>>) -> (Arc<Queue>, Arc<Queue>, Option<Arc<Queue>>) {
	match queues[..] {
		[ref g, ref t, ref c] => (g.clone(), t.clone(), Some(c.clone())),
		[ref gt, ref c] => (gt.clone(), gt.clone(), Some(c.clone())),
		[ref gt] => (gt.clone(), gt.clone(), None),
		_ => unreachable!(),
	}
}

fn try_all_queue_families(physical_device: &PhysicalDevice) -> Option<Vec<QueueFamilyLayout>> {
	queue_families_priorities(
		physical_device,
		vec![
			// main-thread graphics + uploads
			QueueFlags::GRAPHICS | QueueFlags::TRANSFER,
			// capture-thread uploads
			QueueFlags::TRANSFER,
		],
	)
	.or_else(|| {
		queue_families_priorities(
			physical_device,
			vec![
				// main thread graphics
				QueueFlags::GRAPHICS,
				// main thread uploads
				QueueFlags::TRANSFER,
				// capture thread uploads
				QueueFlags::TRANSFER,
			],
		)
	})
	.or_else(|| {
		queue_families_priorities(
			physical_device,
			// main thread-only. software capture not supported.
			vec![QueueFlags::GRAPHICS | QueueFlags::TRANSFER],
		)
	})
}

fn queue_families_priorities(
	physical_device: &PhysicalDevice,
	mut requested_queues: Vec<QueueFlags>,
) -> Option<Vec<QueueFamilyLayout>> {
	let mut result = Vec::with_capacity(3);

	for (idx, props) in physical_device.queue_family_properties().iter().enumerate() {
		let mut remaining = props.queue_count;
		let mut want = 0usize;

		requested_queues.retain(|requested| {
			if props.queue_flags.intersects(*requested) && remaining > 0 {
				remaining -= 1;
				want += 1;
				false
			} else {
				true
			}
		});

		if want > 0 {
			result.push(QueueFamilyLayout {
				queue_family_index: idx as u32,
				priorities: std::iter::repeat_n(1.0, want).collect(),
			});
		}
	}

	if requested_queues.is_empty() {
		log::debug!("Selected GPU queue families: {result:?}");
		Some(result)
	} else {
		None
	}
}
