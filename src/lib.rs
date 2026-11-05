//! # canaima-gpu
//!
//! Módulo de detección de aceleración gráfica y compatibilidad de entorno
//! para Canaima GNU/Linux.
//!
//! Resuelve problemas críticos de renderizado en WebKitGTK (pantalla blanca,
//! congelamiento o `Error 71 Wayland Protocol error`) causados por conflictos
//! con el renderizador DMA-BUF en GPUs NVIDIA y compositores Wayland (KWin / Mutter).

use std::fs;
use std::path::Path;

/// Configuración para la aplicación de parches gráficos.
#[derive(Debug, Clone)]
pub struct GpuOptions {
    /// Forzar desactivación del renderizador DMA-BUF de WebKitGTK.
    pub force_disable_dmabuf: bool,
    /// Forzar desactivación del modo de composición acelerada.
    pub force_disable_compositing: bool,
    /// Forzar desactivación de explicit sync en NVIDIA.
    pub force_disable_explicit_sync: bool,
    /// Emitir mensajes de diagnóstico en la salida estándar.
    pub verbose: bool,
}

impl Default for GpuOptions {
    fn default() -> Self {
        Self {
            force_disable_dmabuf: false,
            force_disable_compositing: false,
            force_disable_explicit_sync: false,
            verbose: cfg!(debug_assertions),
        }
    }
}

/// Detecta si hay un controlador de GPU NVIDIA activo con drivers privativos u open.
#[cfg(target_os = "linux")]
pub fn is_nvidia_active() -> bool {
    // 1. Verificación directa por procfs (módulos privativos u open)
    if Path::new("/proc/driver/nvidia").exists() {
        return true;
    }

    // 2. Verificación por sysfs de módulos del kernel
    if Path::new("/sys/module/nvidia").exists()
        || Path::new("/sys/module/nvidia_drm").exists()
        || Path::new("/sys/module/nvidia_modeset").exists()
    {
        return true;
    }

    // 3. Verificación de nodos de dispositivo de caracteres
    if Path::new("/dev/nvidiactl").exists() || Path::new("/dev/nvidia0").exists() {
        return true;
    }

    // 4. Verificación en /proc/modules si los directorios anteriores tienen permisos restringidos
    if let Ok(modules) = fs::read_to_string("/proc/modules") {
        for line in modules.lines() {
            let mod_name = line.split_whitespace().next().unwrap_or("");
            if mod_name == "nvidia" || mod_name.starts_with("nvidia_") {
                return true;
            }
        }
    }

    // 5. Escaneo exhaustivo en PCI devices (clase 0x03, vendor 0x10de o driver vinculado)
    let pci_dir = Path::new("/sys/bus/pci/devices");
    if let Ok(entries) = fs::read_dir(pci_dir) {
        let found = entries.flatten().any(|entry| {
            let path = entry.path();

            // Verificar si es controlador gráfico (Clase 0x03xxxx: VGA, XGA o 3D Controller)
            let is_gpu = fs::read_to_string(path.join("class"))
                .map(|c| c.trim().to_ascii_lowercase().starts_with("0x03"))
                .unwrap_or(false);

            // Verificar si el fabricante es NVIDIA (Vendor ID: 0x10de)
            let is_nvidia = fs::read_to_string(path.join("vendor"))
                .map(|v| v.trim().eq_ignore_ascii_case("0x10de"))
                .unwrap_or(false);

            // Verificar driver vinculado: soporta nvidia, nvidia-open, nvidia_drm, nvidia-current, etc.
            let uses_nvidia_driver = fs::read_link(path.join("driver"))
                .map(|p| {
                    let dname = p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_ascii_lowercase();
                    dname.starts_with("nvidia")
                })
                .unwrap_or(false);

            is_gpu && (is_nvidia || uses_nvidia_driver)
        });

        if found {
            return true;
        }
    }

    false
}

#[cfg(not(target_os = "linux"))]
pub fn is_nvidia_active() -> bool {
    false
}

/// Detecta si la sesión gráfica actual se está ejecutando sobre Wayland.
#[cfg(target_os = "linux")]
pub fn is_wayland() -> bool {
    let session_type = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_ascii_lowercase();
    session_type == "wayland" || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

#[cfg(not(target_os = "linux"))]
pub fn is_wayland() -> bool {
    false
}

/// Detecta si el entorno de escritorio actual es KDE Plasma.
#[cfg(target_os = "linux")]
pub fn is_kde_plasma() -> bool {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let session = std::env::var("DESKTOP_SESSION")
        .unwrap_or_default()
        .to_ascii_lowercase();
    desktop.contains("kde") || desktop.contains("plasma") || session.contains("plasma")
}

#[cfg(not(target_os = "linux"))]
pub fn is_kde_plasma() -> bool {
    false
}

/// Aplica los parches de compatibilidad recomendados con opciones por defecto.
pub fn apply_workarounds() {
    apply_workarounds_with_options(&GpuOptions::default());
}

/// Aplica los parches de compatibilidad según las opciones especificadas.
#[cfg(target_os = "linux")]
pub fn apply_workarounds_with_options(opts: &GpuOptions) {
    let nvidia = is_nvidia_active();
    let wayland = is_wayland();
    let kde = is_kde_plasma();

    // Los parches automáticos solo deben activarse si hay una GPU NVIDIA activa
    // o si el desarrollador los solicita explícitamente, preservando la aceleración
    // por hardware en compositores Wayland y KDE para GPUs Intel y AMD.
    let should_disable_dmabuf = opts.force_disable_dmabuf || nvidia;
    let should_disable_compositing =
        opts.force_disable_compositing || (nvidia && (kde || wayland));
    let should_disable_explicit_sync =
        opts.force_disable_explicit_sync || (nvidia && wayland);

    if opts.verbose {
        println!(
            "🔧 [canaima-gpu] Diagnóstico de entorno -> NVIDIA: {}, Wayland: {}, KDE: {}",
            nvidia, wayland, kde
        );
    }

    unsafe {
        // 1. WEBKIT_DISABLE_DMABUF_RENDERER: Soluciona el error fatal de protocolo y pantallas en blanco
        if should_disable_dmabuf && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            if opts.verbose {
                println!("🔧 [canaima-gpu] Activando WEBKIT_DISABLE_DMABUF_RENDERER=1");
            }
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }

        // 2. WEBKIT_DISABLE_COMPOSITING_MODE: Evita bloqueos en drivers privativos y KDE KWin
        if should_disable_compositing
            && std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none()
        {
            if opts.verbose {
                println!("🔧 [canaima-gpu] Activando WEBKIT_DISABLE_COMPOSITING_MODE=1");
            }
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }

        // 3. __NV_DISABLE_EXPLICIT_SYNC: Previene Protocol Error 71 en NVIDIA + Wayland
        if should_disable_explicit_sync
            && std::env::var_os("__NV_DISABLE_EXPLICIT_SYNC").is_none()
        {
            if opts.verbose {
                println!("🔧 [canaima-gpu] Activando __NV_DISABLE_EXPLICIT_SYNC=1");
            }
            std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn apply_workarounds_with_options(_opts: &GpuOptions) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = GpuOptions::default();
        assert!(!opts.force_disable_dmabuf);
        assert!(!opts.force_disable_compositing);
        assert!(!opts.force_disable_explicit_sync);
    }

    #[test]
    fn test_detection_functions_do_not_panic() {
        let _ = is_nvidia_active();
        let _ = is_wayland();
        let _ = is_kde_plasma();
    }

    #[test]
    fn test_apply_workarounds_with_options_does_not_panic() {
        let opts = GpuOptions {
            force_disable_dmabuf: false,
            force_disable_compositing: false,
            force_disable_explicit_sync: false,
            verbose: false,
        };
        apply_workarounds_with_options(&opts);
    }
}


