
use std::sync::Arc;

use crate::{BackendError, Result};

#[derive(Debug, Clone)]
pub enum GpuBackend {
    Vulkan,
    Metal,
    Dx12,
    Gl,
    BrowserWebGpu,
    Cpu,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct GpuProperties {
    pub device_id: usize,
    pub name: String,
    pub backend: GpuBackend,
    pub vendor: String,
    pub device_type: String,
    pub vram_total_mb: usize,
    pub compute_units: u32,
    pub max_workgroup: u32,
}

pub struct GpuDevice {
    pub props: GpuProperties,

    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl GpuDevice {
    pub fn open_best() -> Result<Self> {
        pollster::block_on(Self::open_best_async())
    }

    pub async fn open_best_async() -> Result<Self> {
        let instance = wgpu::Instance::default();

        let adapters = instance.enumerate_adapters(
            wgpu::Backends::all()
        );

        let mut best = None;

        for adapter in adapters {
            let info = adapter.get_info();

            if info.device_type == wgpu::DeviceType::Cpu {
                continue;
            }

            let score = match info.device_type {
                wgpu::DeviceType::DiscreteGpu => 4,
                wgpu::DeviceType::IntegratedGpu => 3,
                wgpu::DeviceType::VirtualGpu => 2,
                _ => 1,
            };

            if best.as_ref().map_or(true, |(_, previous_score)| score > *previous_score) {
                best = Some((adapter, score));
            }
        }

        let (adapter, _) = best.ok_or_else(|| {
            BackendError::DeviceNotFound
        })?;

        Self::from_adapter(adapter).await
    }

    async fn from_adapter(
        adapter: wgpu::Adapter
    ) -> Result<Self> {

        let info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GPU Device"),
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let backend = match info.backend {
            wgpu::Backend::Vulkan => GpuBackend::Vulkan,
            wgpu::Backend::Metal => GpuBackend::Metal,
            wgpu::Backend::Dx12 => GpuBackend::Dx12,
            wgpu::Backend::Gl => GpuBackend::Gl,
            _ => GpuBackend::Unknown,
        };

        Ok(Self {
            props: GpuProperties {
                device_id: 0,
                name: info.name,
                backend,
                vendor: format!("{:04x}", info.vendor),
                device_type: format!("{:?}", info.device_type),
                vram_total_mb: 0,
                compute_units: 0,
                max_workgroup:
                    device.limits().max_compute_workgroup_size_x,
            },

            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    pub fn synchronise(&self) {
        let _ = self.device.poll(wgpu::Maintain::Wait);
    }
}
