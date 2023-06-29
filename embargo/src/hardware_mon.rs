use std::time::{Duration, Instant};
use sysinfo::{CpuExt, NetworkExt, SystemExt};

use sysinfo::System;

pub struct HardwareMonitor {
    system: System,
    modules: Vec<HardwareModule>,
    interface_name: String,
    network_refresh_frequency: Duration,
}

impl HardwareMonitor {
    pub fn new(interface_name: String) -> Self {
        let network_refresh_frequency = Duration::from_secs(2);
        let modules = vec![
            HardwareModule::new(Duration::from_millis(500), Box::new(System::refresh_cpu)),
            HardwareModule::new(
                Duration::from_millis(1000),
                Box::new(System::refresh_memory),
            ),
            HardwareModule::new(
                network_refresh_frequency,
                Box::new(System::refresh_networks),
            ),
            HardwareModule::new(
                Duration::from_secs(10),
                Box::new(System::refresh_networks_list),
            ),
        ];
        Self {
            system: System::new(),
            modules,
            network_refresh_frequency,
            interface_name,
        }
    }
    pub fn update(&mut self) {
        for module in &mut self.modules {
            module.update(&mut self.system);
        }
    }
    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage()
    }
    pub fn used_mem(&self) -> u64 {
        self.system.used_memory()
    }
    pub fn total_mem(&self) -> u64 {
        self.system.total_memory()
    }
    pub fn uploaded_bytes(&self) -> u64 {
        if let Some(bytes) = self
            .system
            .networks()
            .into_iter()
            .find(|(name, _)| **name == self.interface_name)
            .map(|(_, network)| network.transmitted())
        {
            bytes / self.network_refresh_frequency.as_secs()
        } else {
            tracing::warn!("unable to locate interface '{}'", self.interface_name);
            0
        }
    }
    pub fn downloaded_bytes(&self) -> u64 {
        if let Some(bytes) = self
            .system
            .networks()
            .into_iter()
            .find(|(name, _)| **name == self.interface_name)
            .map(|(_, network)| network.received())
        {
            bytes / self.network_refresh_frequency.as_secs()
        } else {
            tracing::warn!("unable to locate interface '{}'", self.interface_name);
            0
        }
    }
}

pub struct HardwareModule {
    last_update: Instant,
    frequency: Duration,
    refresh: Box<dyn Fn(&mut System)>,
}

impl HardwareModule {
    pub fn new(frequency: Duration, refresh: Box<dyn Fn(&mut System)>) -> Self {
        Self {
            last_update: Instant::now().checked_sub(frequency).unwrap(),
            frequency,
            refresh,
        }
    }
    pub fn update(&mut self, system: &mut System) {
        if self.last_update.elapsed() > self.frequency {
            self.last_update = Instant::now();
            (self.refresh)(system);
        }
    }
}
