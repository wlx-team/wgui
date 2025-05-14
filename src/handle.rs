#[macro_export]
macro_rules! gen_id {
	(
		$container_name:ident,
		$instance_name:ident,
		$cell_name:ident,
		$handle_name:ident) => {
		//ThingCell
		pub struct $cell_name {
			pub obj: $instance_name,
			generation: u64,
		}

		//ThingVec
		pub struct $container_name {
			// Vec<Option<ThingCell>>
			pub vec: Vec<Option<$cell_name>>,

			cur_generation: u64,
			count: u32,
		}

		//ThingHandle
		#[derive(Default, Clone, Copy, PartialEq, Hash, Eq)]
		pub struct $handle_name {
			pub idx: u32,
			pub generation: u64,
		}

		#[allow(dead_code)]
		impl $handle_name {
			pub fn reset(&mut self) {
				self.generation = 0;
			}

			pub fn is_set(&self) -> bool {
				self.generation > 0
			}

			pub fn id(&self) -> u32 {
				self.idx
			}
		}

		//ThingVec
		#[allow(dead_code)]
		impl $container_name {
			pub fn new() -> Self {
				Self {
					vec: Vec::new(),
					cur_generation: 0,
					count: 0,
				}
			}

			pub fn count(&self) -> u32 {
				self.count
			}

			pub fn iter(&self) -> impl Iterator<Item = ($handle_name, &$instance_name)> {
				self.vec.iter().enumerate().filter_map(|(idx, opt_cell)| {
					opt_cell.as_ref().map(|cell| {
						let handle = $container_name::get_handle(&cell, idx);
						(handle, &cell.obj)
					})
				})
			}

			pub fn find(
				&self,
				pred: &mut dyn FnMut($handle_name, &$instance_name) -> bool,
			) -> Option<$handle_name> {
				for (idx, opt_cell) in self.vec.iter().enumerate() {
					if let Some(cell) = opt_cell {
						let handle = $container_name::get_handle(&cell, idx);
						if pred(handle, &cell.obj) {
							return Some(handle);
						}
					}
				}
				None
			}

			pub fn iter_mut(&mut self) -> impl Iterator<Item = ($handle_name, &mut $instance_name)> {
				self
					.vec
					.iter_mut()
					.enumerate()
					.filter_map(|(idx, opt_cell)| {
						opt_cell.as_mut().map(|cell| {
							let handle = $container_name::get_handle(&cell, idx);
							(handle, &mut cell.obj)
						})
					})
			}

			pub fn get_handle(cell: &$cell_name, idx: usize) -> $handle_name {
				$handle_name {
					idx: idx as u32,
					generation: cell.generation,
				}
			}

			fn find_unused_idx(&mut self) -> Option<u32> {
				for (num, obj) in self.vec.iter().enumerate() {
					if obj.is_none() {
						return Some(num as u32);
					}
				}
				None
			}

			fn add_alloc_handle(&mut self) -> ($handle_name, Option<u32> /* unused idx */) {
				self.cur_generation += 1;
				let unused_idx = self.find_unused_idx();

				let idx = if let Some(idx) = unused_idx {
					idx
				} else {
					self.vec.len() as u32
				};

				let handle = $handle_name {
					idx,
					generation: self.cur_generation,
				};

				(handle, unused_idx)
			}

			fn add_ret(
				&mut self,
				handle: $handle_name,
				unused_idx: Option<u32>,
				obj: $instance_name,
			) -> $handle_name {
				let cell = $cell_name {
					obj,
					generation: self.cur_generation,
				};

				if let Some(idx) = unused_idx {
					self.vec[idx as usize] = Some(cell);
				} else {
					self.vec.push(Some(cell))
				}

				self.count += 1;

				handle
			}

			pub fn add(&mut self, obj: $instance_name) -> $handle_name {
				let (handle, unused_idx) = self.add_alloc_handle();
				self.add_ret(handle, unused_idx, obj)
			}

			pub fn add_with_post<F>(&mut self, mut obj: $instance_name, callback: F) -> $handle_name
			where
				F: FnOnce($handle_name, &mut $instance_name),
			{
				let (handle, unused_idx) = self.add_alloc_handle();
				callback(handle, &mut obj);
				self.add_ret(handle, unused_idx, obj)
			}

			fn shrink(&mut self) {
				while let Some(last) = self.vec.last() {
					if last.is_none() {
						self.vec.pop();
						continue;
					}
					break;
				}
			}

			pub fn remove(&mut self, handle: &$handle_name) {
				// Out of bounds, ignore
				if handle.idx as usize >= self.vec.len() {
					return;
				}

				// Remove only if the generation matches
				if let Some(cell) = &self.vec[handle.idx as usize] {
					if cell.generation == handle.generation {
						self.vec[handle.idx as usize] = None;
						self.count -= 1;
						self.shrink();
					}
				}
			}

			pub fn get(&self, handle: &$handle_name) -> Option<&$instance_name> {
				// Out of bounds, ignore
				if handle.idx as usize >= self.vec.len() {
					return None;
				}

				if let Some(cell) = &self.vec[handle.idx as usize] {
					if cell.generation == handle.generation {
						return Some(&cell.obj);
					}
				}

				None
			}

			pub fn get_mut(&mut self, handle: &$handle_name) -> Option<&mut $instance_name> {
				// Out of bounds, ignore
				if handle.idx as usize >= self.vec.len() {
					return None;
				}

				if let Some(cell) = &mut self.vec[handle.idx as usize] {
					if cell.generation == handle.generation {
						return Some(&mut cell.obj);
					}
				}

				None
			}
		}
	};
}
