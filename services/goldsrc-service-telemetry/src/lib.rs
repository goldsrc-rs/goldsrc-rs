//! Host-side hardware metrics and system diagnostics service for GoldSrc.rs.
//!
//! Provides telemetry collection via `sysinfo` for CPU, process/system memory,
//! and engine tickrate/jitter monitoring.

use std::sync::Mutex;
use std::time::Instant;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

/// Snapshot of host hardware and process performance metrics.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemMetricsSnapshot {
    /// Host CPU brand / model name (e.g. "Intel Core i7-10700K").
    pub cpu_brand: String,
    /// Host CPU vendor identifier (e.g. "GenuineIntel").
    pub cpu_vendor: String,
    /// Physical core count if detected.
    pub physical_cores: Option<usize>,
    /// Logical core (thread) count.
    pub logical_cores: usize,
    /// Global CPU utilization percentage (0.0 to 100.0).
    pub global_cpu_usage: f32,
    /// Process-specific CPU utilization percentage.
    pub process_cpu_usage: f32,
    /// Resident Set Size (RSS) memory used by the HLDS process in bytes.
    pub process_memory_rss_bytes: u64,
    /// Virtual memory used by the HLDS process in bytes.
    pub process_memory_virtual_bytes: u64,
    /// Total physical RAM on the host in bytes.
    pub total_memory_bytes: u64,
    /// Currently available physical RAM on the host in bytes.
    pub available_memory_bytes: u64,
    /// Total swap memory in bytes.
    pub total_swap_bytes: u64,
    /// Used swap memory in bytes.
    pub used_swap_bytes: u64,
    /// Measured engine tickrate in frames per second (FPS).
    pub measured_fps: f64,
    /// Frame time jitter standard deviation or range in milliseconds.
    pub frame_jitter_ms: f64,
    /// Uptime of the GoldSrc.rs runtime in seconds.
    pub uptime_secs: u64,
}

/// Rolling window frame time tracker for jitter and tickrate stability measurement.
#[derive(Debug)]
pub struct FrameTimeTracker {
    last_frame_instant: Option<Instant>,
    samples: Vec<f64>,
    sample_capacity: usize,
    sample_index: usize,
    total_samples: usize,
}

impl FrameTimeTracker {
    /// Creates a new frame time tracker with the specified sample buffer capacity.
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(16);
        Self {
            last_frame_instant: None,
            samples: vec![0.0; cap],
            sample_capacity: cap,
            sample_index: 0,
            total_samples: 0,
        }
    }

    /// Records the passage of an engine frame tick.
    pub fn record_frame(&mut self, now: Instant) {
        if let Some(prev) = self.last_frame_instant {
            let dt = now.duration_since(prev).as_secs_f64();
            if dt > 0.0 && dt < 2.0 {
                // Filter absurd delta spikes during level changes or debug freezes
                self.samples[self.sample_index] = dt;
                self.sample_index = (self.sample_index + 1) % self.sample_capacity;
                if self.total_samples < self.sample_capacity {
                    self.total_samples += 1;
                }
            }
        }
        self.last_frame_instant = Some(now);
    }

    /// Computes current average FPS and jitter (standard deviation of frame times in ms).
    pub fn calculate_fps_and_jitter(&self) -> (f64, f64) {
        if self.total_samples < 2 {
            return (0.0, 0.0);
        }
        let count = self.total_samples;
        let mut sum = 0.0;
        for i in 0..count {
            sum += self.samples[i];
        }
        let mean_dt = sum / (count as f64);
        if mean_dt <= 0.0 {
            return (0.0, 0.0);
        }
        let fps = 1.0 / mean_dt;

        // Compute standard deviation (jitter) in milliseconds
        let mut variance_sum = 0.0;
        for i in 0..count {
            let diff = (self.samples[i] - mean_dt) * 1000.0; // convert to ms
            variance_sum += diff * diff;
        }
        let jitter_ms = (variance_sum / (count as f64)).sqrt();
        (fps, jitter_ms)
    }
}

/// Host System Information Service collecting CPU, RAM, and tickrate telemetry.
pub struct SystemInfoService {
    system: Mutex<System>,
    pid: Pid,
    start_time: Instant,
    frame_tracker: Mutex<FrameTimeTracker>,
}

impl Default for SystemInfoService {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoService {
    /// Creates and initializes a new `SystemInfoService`.
    pub fn new() -> Self {
        let refresh_kind = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        let mut system = System::new_with_specifics(refresh_kind);
        system.refresh_cpu_all();
        system.refresh_memory();

        let pid = Pid::from_u32(std::process::id());
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );

        Self {
            system: Mutex::new(system),
            pid,
            start_time: Instant::now(),
            frame_tracker: Mutex::new(FrameTimeTracker::new(128)),
        }
    }

    /// Notifies the service that a new frame has completed. Call from engine frame hook (`StartFrame`).
    pub fn on_frame(&self) {
        if let Ok(mut tracker) = self.frame_tracker.lock() {
            tracker.record_frame(Instant::now());
        }
    }

    /// Captures a fresh snapshot of all system and process telemetry.
    pub fn sample_metrics(&self) -> SystemMetricsSnapshot {
        let mut sys = self.system.lock().unwrap_or_else(|p| p.into_inner());

        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[self.pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );

        let cpus = sys.cpus();
        let (cpu_brand, cpu_vendor) = if let Some(cpu) = cpus.first() {
            (cpu.brand().to_string(), cpu.vendor_id().to_string())
        } else {
            ("Unknown CPU".to_string(), "Unknown Vendor".to_string())
        };

        let physical_cores = sysinfo::System::physical_core_count();
        let logical_cores = cpus.len();
        let global_cpu_usage = sys.global_cpu_usage();

        let (proc_cpu, proc_rss, proc_virt) = if let Some(proc) = sys.process(self.pid) {
            (proc.cpu_usage(), proc.memory(), proc.virtual_memory())
        } else {
            (0.0, 0, 0)
        };

        let total_mem = sys.total_memory();
        let avail_mem = sys.available_memory();
        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();

        let (measured_fps, frame_jitter_ms) = self
            .frame_tracker
            .lock()
            .map(|t| t.calculate_fps_and_jitter())
            .unwrap_or((0.0, 0.0));

        let uptime_secs = self.start_time.elapsed().as_secs();

        SystemMetricsSnapshot {
            cpu_brand,
            cpu_vendor,
            physical_cores,
            logical_cores,
            global_cpu_usage,
            process_cpu_usage: proc_cpu,
            process_memory_rss_bytes: proc_rss,
            process_memory_virtual_bytes: proc_virt,
            total_memory_bytes: total_mem,
            available_memory_bytes: avail_mem,
            total_swap_bytes: total_swap,
            used_swap_bytes: used_swap,
            measured_fps,
            frame_jitter_ms,
            uptime_secs,
        }
    }
}

static SYSTEM_INFO_SERVICE: std::sync::OnceLock<SystemInfoService> = std::sync::OnceLock::new();

/// Global singleton access to the host `SystemInfoService`.
pub fn system_info() -> &'static SystemInfoService {
    SYSTEM_INFO_SERVICE.get_or_init(SystemInfoService::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_system_info_snapshot_contains_valid_hardware_data() {
        let service = SystemInfoService::new();
        // Give sysinfo a brief sleep or repeated sample for cpu calculation
        std::thread::sleep(Duration::from_millis(10));
        let snapshot = service.sample_metrics();

        assert!(!snapshot.cpu_brand.is_empty() || snapshot.logical_cores > 0);
        assert!(snapshot.total_memory_bytes > 0);
    }

    #[test]
    fn test_frame_tracker_fps_and_jitter_calculation() {
        let mut tracker = FrameTimeTracker::new(32);
        let start = Instant::now();

        // Simulate 20 frames with ~10ms interval (100 FPS)
        for i in 0..20 {
            let t = start + Duration::from_millis(i * 10);
            tracker.record_frame(t);
        }

        let (fps, jitter) = tracker.calculate_fps_and_jitter();
        assert!(
            (90.0..=110.0).contains(&fps),
            "Expected ~100 FPS, got {fps}"
        );
        assert!(
            (0.0..1.0).contains(&jitter),
            "Expected low jitter, got {jitter}ms"
        );
    }
}
