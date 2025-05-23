pub mod vert_atlas {
	vulkano_shaders::shader! {
			ty: "vertex",
			path: "src/renderers/text/text.vert",
	}
}

pub mod frag_atlas {
	vulkano_shaders::shader! {
			ty: "fragment",
			path: "src/renderers/text/text.frag",
	}
}
