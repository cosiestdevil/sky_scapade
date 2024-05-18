use bevy::prelude::*;
use bevy_ecs::{prelude::ResMut, system::Local};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use bevy_diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore};

const BYTES_TO_GIB: f64 = 1.0 / 1024.0 / 1024.0 / 1024.0;

#[derive(Default)]
pub struct SystemInformationDiagnosticsPlugin;
impl Plugin for SystemInformationDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_system)
            .add_systems(FixedUpdate, diagnostic_system);
    }
}

impl SystemInformationDiagnosticsPlugin {
    pub const CPU_USAGE: DiagnosticPath = DiagnosticPath::const_new("system/cpu_usage");
    pub const MEM_USAGE: DiagnosticPath = DiagnosticPath::const_new("system/mem_usage");
}

pub(crate) fn setup_system(mut diagnostics: ResMut<DiagnosticsStore>) {
    diagnostics
        .add(Diagnostic::new(SystemInformationDiagnosticsPlugin::CPU_USAGE).with_suffix("%"));
    diagnostics
        .add(Diagnostic::new(SystemInformationDiagnosticsPlugin::MEM_USAGE).with_suffix("%"));
}

pub(crate) fn diagnostic_system(
    mut diagnostics: Diagnostics,
    mut sysinfo: Local<Option<System>>,
) {
    if sysinfo.is_none() {
        *sysinfo = Some(System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                .with_memory(MemoryRefreshKind::everything()),
        ));
    }
    let Some(sys) = sysinfo.as_mut() else {
        return;
    };

    sys.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());
    sys.refresh_memory();
    let current_cpu_usage = sys.global_cpu_info().cpu_usage();
    // `memory()` fns return a value in bytes
    let total_mem = sys.total_memory() as f64 / BYTES_TO_GIB;
    let used_mem = sys.used_memory() as f64 / BYTES_TO_GIB;
    let current_used_mem = used_mem / total_mem * 100.0;

    diagnostics.add_measurement(&SystemInformationDiagnosticsPlugin::CPU_USAGE, || {
        current_cpu_usage as f64
    });
    diagnostics.add_measurement(&SystemInformationDiagnosticsPlugin::MEM_USAGE, || {
        current_used_mem
    });
}
