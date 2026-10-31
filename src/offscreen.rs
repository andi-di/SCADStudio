//! Display-independent wgpu proof, using eframe's exact wgpu/egui renderer versions.
use crate::render::{Mesh, shapes};
use anyhow::{Context, Result};
use eframe::{egui, egui_wgpu, wgpu};
use std::{fs::File, path::Path, sync::mpsc};

/// Writes RGBA8 bytes to a local PNG; call outside the interactive UI thread.
pub fn save_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<()> {
    let mut encoder = png::Encoder::new(File::create(path)?, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(rgba)?;
    Ok(())
}

/// Renders real evaluated geometry into a texture, reads it back and writes PNG.
/// Does not create a window, surface or display connection. Returns adapter evidence.
pub fn capture(mesh: &Mesh, path: &Path) -> Result<String> {
    pollster::block_on(capture_async(mesh, path))
}

async fn capture_async(mesh: &Mesh, path: &Path) -> Result<String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: None,
            force_fallback_adapter: false,
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        })
        .await
        .context("offscreen wgpu adapter unavailable")?;
    let info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await?;
    let (width, height) = (640, 480);
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("M0 headless view"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let context = egui::Context::default();
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width as f32, height as f32));
    let mut output = context.run_pass(
        egui::RawInput {
            screen_rect: Some(rect),
            ..Default::default()
        },
        |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::background());
            if let Some(bounds) = mesh.bounds() {
                painter.extend(shapes(mesh, bounds, rect, -0.6, false));
            }
        },
    );
    let primitives = context.tessellate(output.shapes, 1.0);
    let mut renderer =
        egui_wgpu::Renderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, Default::default());
    for (id, delta) in &output.textures_delta.set {
        for image_delta in delta {
            renderer.update_texture(&device, &queue, *id, image_delta);
        }
    }
    let screen = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [width, height],
        pixels_per_point: 1.0,
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    let extra = renderer.update_buffers(&device, &queue, &mut encoder, &primitives, &screen);
    {
        let view = texture.create_view(&Default::default());
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("M0 offscreen"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.09,
                        g: 0.11,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        renderer.render(&mut pass.forget_lifetime(), &primitives, &screen);
    }
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("M0 readback"),
        size: u64::from(width * height * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
        },
        size,
    );
    queue.submit(extra.into_iter().chain([encoder.finish()]));
    let (send, receive) = mpsc::sync_channel(1);
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
    device.poll(wgpu::PollType::wait_indefinitely())?;
    receive.recv()??;
    save_png(path, width, height, &buffer.slice(..).get_mapped_range()?)?;
    buffer.unmap();
    for id in &output.textures_delta.free {
        renderer.free_texture(id);
    }
    output.textures_delta.clear();
    Ok(format!(
        "{} / {:?} / {}",
        info.name, info.backend, info.driver_info
    ))
}
