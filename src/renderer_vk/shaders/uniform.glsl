
// Viewport
layout(set = UNIFORM_PARAMS_SET, binding = 0) uniform UniformParams {
  uniform uvec2 screen_resolution;
  uniform mat4 models[64]; // ModelBuffer::MAX_COUNT
}
uniforms;