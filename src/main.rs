use rt_cpu::render_cpu;
use spirv_std::glam::uvec2;

pub fn main() {
    render_cpu(uvec2(1920, 1080));
}
