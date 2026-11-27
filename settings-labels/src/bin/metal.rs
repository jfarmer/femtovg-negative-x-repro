use femtovg::renderer::WGPURenderer;
fn read_pixels(device: &wgpu::Device, queue: &wgpu::Queue, target: &wgpu::Texture) -> Vec<u8> {
    let (width, height) = (900u32, 900u32);
    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("settings-labels readback"),
        size: u64::from(padded * height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).expect("device lost");
    let mapped = slice.get_mapped_range().expect("readback mapped");
    mapped
        .chunks(padded as usize)
        .flat_map(|row| &row[..unpadded as usize])
        .copied()
        .collect()
}
fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();

    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .map_err(|e| format!("no GPU adapter: {e}"))?;
    assert_eq!(adapter.get_info().backend, wgpu::Backend::Metal);
    let limits = wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits());
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: limits,
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        ..Default::default()
    }))
    .map_err(|e| format!("no GPU device: {e}"))?;

    let size = (900u32, 900u32);
    let ctx = femtovg::TextContext::default();
    let mut canvas =
        femtovg::Canvas::new_with_text_context(WGPURenderer::new(device.clone(), queue.clone()), ctx.clone()).unwrap();
    canvas.set_size(900, 900, 1.);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("settings-labels target"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let scene = settings_labels::Scene::new(&ctx);
    settings_labels::measure(
        &mut canvas,
        &scene,
        |c| c.flush_to_output(&target),
        |cmds| {
            queue.submit(cmds);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        },
    );
    if args.len() > 2 {
        std::fs::write(&args[2], read_pixels(&device, &queue, &target)).unwrap();
    }
    Ok(())
}
