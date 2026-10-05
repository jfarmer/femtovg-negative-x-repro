//! A femtovg canvas on `renderer::WGPURenderer` that draws to a texture instead of a window, and
//! the code that reads the finished frame back as an image.

use femtovg::{renderer::WGPURenderer, Canvas, TextContext};
use image::RgbaImage;

pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// The name and backend of the adapter wgpu chose. compose.rs does not print it.
    #[allow(dead_code)]
    pub adapter_name: String,
}

impl Gpu {
    pub fn new() -> Gpu {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }))
        .expect("no wgpu adapter");
        let info = adapter.get_info();
        let adapter_name = format!("{} ({:?})", info.name, info.backend);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::default(),
        }))
        .expect("no wgpu device");
        device.on_uncaptured_error(std::sync::Arc::new(|error| panic!("wgpu error: {error}")));
        Gpu {
            device,
            queue,
            adapter_name,
        }
    }

    /// A canvas of `width` x `height` device pixels at device pixel ratio 1.
    pub fn canvas(&self, text_context: TextContext, width: u32, height: u32) -> Canvas<WGPURenderer> {
        let renderer = WGPURenderer::new(self.device.clone(), self.queue.clone());
        let mut canvas = Canvas::new_with_text_context(renderer, text_context).expect("canvas");
        canvas.set_size(width, height, 1.0);
        canvas
    }

    /// Flushes `canvas` to a new texture of the given size and returns its pixels.
    pub fn read_frame(&self, canvas: &mut Canvas<WGPURenderer>, width: u32, height: u32) -> RgbaImage {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let commands = canvas
            .flush_to_output(&target)
            .expect("flush_to_output returned no commands");

        // wgpu requires the rows of the buffer a texture is copied to to be padded to a multiple
        // of COPY_BYTES_PER_ROW_ALIGNMENT.
        let row = width * 4;
        let padded_row = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(padded_row * height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(height),
                },
            },
            extent,
        );
        self.queue.submit([commands, encoder.finish()]);

        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).expect("the receiver of the map result was dropped");
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll failed");
        receiver
            .recv()
            .expect("the map_async callback did not run")
            .expect("mapping the readback buffer failed");
        let mapped = slice.get_mapped_range().expect("mapped range of the readback buffer");
        let mut pixels = Vec::with_capacity((row * height) as usize);
        for padded in mapped.chunks_exact(padded_row as usize) {
            pixels.extend_from_slice(&padded[..row as usize]);
        }
        drop(mapped);
        readback.unmap();
        RgbaImage::from_raw(width, height, pixels).expect("frame size")
    }
}
