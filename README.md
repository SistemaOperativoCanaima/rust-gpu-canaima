# 🚀 canaima-gpu (`lib-rust-gpu`)

<div align="center">

[![Canaima GNU/Linux](https://img.shields.io/badge/Canaima-GNU%2FLinux-0B6793?style=for-the-badge&logo=linux&logoColor=white)](https://canaima.softwarelibre.gob.ve/)
[![Rust 2021](https://img.shields.io/badge/Rust-2021%20Edition-black?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)]()
[![Tauri v2](https://img.shields.io/badge/Tauri-v2.x-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)

**Detección inteligente de hardware gráfico y parches de compatibilidad WebKitGTK / Wayland / X11 para Canaima GNU/Linux.**

</div>

---

## 📖 Descripción General

**`canaima-gpu`** es una librería en Rust diseñada específicamente para resolver problemas críticos de renderizado y estabilidad que experimentan las aplicaciones de escritorio basadas en **WebKitGTK** y **Tauri 2** en entornos GNU/Linux.

En particular, previene:
- ❌ **Pantallas blancas o congelamientos** al iniciar la aplicación.
- ❌ **`Wayland Protocol Error 71`** ocasionado por fallos de sincronización explícita (`explicit sync`) en controladores NVIDIA.
- ❌ Conflictos con el renderizador **DMA-BUF** en compositores Wayland y compositores X11 ligeros.
- ❌ Degradación de rendimiento en tarjetas **Intel** o **AMD**, asegurando que la aceleración por hardware *zero-copy* se mantenga activa en usuarios que no usan NVIDIA.

---

## ✨ Características Principales

- ⚡ **Sin llamadas lentas al sistema**: No depende de comandos externos como `lspci` ni `nvidia-smi`. Inspecciona directamente `/sys`, `/proc` y las interfaces de caracteres `/dev/nvidia*` para una inicialización instantánea (< 1 ms).
- 🎯 **Detección multinivel de NVIDIA**:
  1. Sistema de archivos `/proc/driver/nvidia` (módulos privativos u open del kernel).
  2. Nodos `/sys/module/nvidia*`.
  3. Dispositivos de caracteres `/dev/nvidiactl` y `/dev/nvidia0`.
  4. Módulos activos en `/proc/modules`.
  5. Escaneo exhaustivo del bus PCI (`/sys/bus/pci/devices`) filtrando por clase gráfica (`0x03xxxx`), Vendor ID NVIDIA (`0x10de`) o symlink activo al controlador del kernel.
- 🖥️ **Soporte completo para 7 entornos de escritorio**: Detecta y se adapta automáticamente a **GNOME**, **KDE Plasma**, **XFCE**, **LXDE**, **MATE**, **Cinnamon** y **LXQt**.
- 🛡️ **Parches selectivos no invasivos**: No desactiva la aceleración por hardware a usuarios con GPUs Intel (Arc / Iris / UHD) ni AMD (Radeon), aplicando parches estrictamente cuando hay GPUs NVIDIA o cuando se solicita explícitamente.
- 🌐 **Zero Dependencies**: Implementado usando exclusivamente la biblioteca estándar de Rust (`std`).

---

## 🖥️ Matriz de Compatibilidad por Entorno

| Entorno de Escritorio | Gestor de Ventanas | Sesión Soportada | Comportamiento del Parche |
| :--- | :--- | :--- | :--- |
| **GNOME** | Mutter | Wayland & X11 | Activa `WEBKIT_DISABLE_DMABUF_RENDERER` y `__NV_DISABLE_EXPLICIT_SYNC` en NVIDIA. |
| **KDE Plasma** | KWin | Wayland & X11 | Previene *Protocol Error 71* y bloqueos en KWin Wayland con NVIDIA. |
| **XFCE** | Xfwm4 | X11 | Desactiva DMA-BUF problemático en X11 sobre NVIDIA, aceleración activa en Intel/AMD. |
| **LXDE** | Openbox | X11 | Modo de composición seguro en NVIDIA para evitar fallos sin compositor. |
| **MATE** | Marco | X11 | Compatible con Marco en modo software o composición GL. |
| **Cinnamon** | Muffin | X11 & Wayland | Parches automáticos aplicados a compositores derivados de Mutter. |
| **LXQt** | Openbox / KWin | X11 & Wayland | Aceleración completa en Intel/AMD y compatibilidad asegurada en NVIDIA. |

---

## 📦 Instalación

Agrega `canaima-gpu` a tu archivo `Cargo.toml`:

```toml
[dependencies]
canaima-gpu = { path = "ruta/a/lib-rust-gpu-main" }
```

O si utilizas un repositorio Git:

```toml
[dependencies]
canaima-gpu = { git = "https://github.com/SistemaOperativoCanaima/rust-gpu-canaima.git", branch = "main" }
```

---

## 💻 Guía de Uso

### Uso Básico Recomendado (Tauri v2 / WebKitGTK)

> [!IMPORTANT]
> `canaima_gpu::apply_workarounds()` debe ejecutarse al **inicio absoluto de la función `main()`**, antes de inicializar hilos adicionales o iniciar el runtime de Tauri / GTK.

En `src-tauri/src/main.rs`:

```rust
// Evita la consola de Windows en modo release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 1. Aplicar parches gráficos de compatibilidad Canaima
    canaima_gpu::apply_workarounds();

    // 2. Iniciar la aplicación Tauri
    mi_app_lib::run();
}
```

---

### Uso Avanzado con Opciones Personalizadas (`GpuOptions`)

Puedes personalizar la activación de los parches utilizando la estructura `GpuOptions`:

```rust
use canaima_gpu::{apply_workarounds_with_options, GpuOptions};

fn main() {
    let opciones = GpuOptions {
        // Forzar desactivación de DMA-BUF sin importar el hardware
        force_disable_dmabuf: false,
        // Forzar desactivación del modo de composición
        force_disable_compositing: false,
        // Forzar desactivación de explicit sync
        force_disable_explicit_sync: false,
        // Mostrar logs detallados de diagnóstico en la terminal
        verbose: true,
    };

    apply_workarounds_with_options(&opciones);

    mi_app_lib::run();
}
```

---

### Consultas de Entorno y Detección en Código

Puedes usar las funciones de detección de forma independiente en tu lógica de negocio:

```rust
use canaima_gpu::{
    is_nvidia_active, is_wayland, get_desktop_environment, DesktopEnvironment
};

fn verificar_sistema() {
    if is_nvidia_active() {
        println!("🎮 GPU NVIDIA activa detectada.");
    }

    if is_wayland() {
        println!("🚀 Sesión sobre Wayland.");
    } else {
        println!("🖥️ Sesión sobre X11.");
    }

    match get_desktop_environment() {
        DesktopEnvironment::Gnome => println!("Entorno: GNOME"),
        DesktopEnvironment::KdePlasma => println!("Entorno: KDE Plasma"),
        DesktopEnvironment::Xfce => println!("Entorno: XFCE"),
        DesktopEnvironment::Cinnamon => println!("Entorno: Cinnamon"),
        DesktopEnvironment::Mate => println!("Entorno: MATE"),
        DesktopEnvironment::Lxqt => println!("Entorno: LXQt"),
        DesktopEnvironment::Lxde => println!("Entorno: LXDE"),
        DesktopEnvironment::Other => println!("Otro entorno de escritorio"),
    }
}
```

---

## ⚙️ Variables de Entorno Gestionadas

Cuando se detecta una GPU NVIDIA (o se fuerza en las opciones), la librería configura automáticamente:

| Variable | Valor | Propósito |
| :--- | :---: | :--- |
| `WEBKIT_DISABLE_DMABUF_RENDERER` | `1` | Desactiva el renderizador DMA-BUF de WebKitGTK para prevenir pantallas blancas y caídas en compositores Wayland. |
| `WEBKIT_DISABLE_COMPOSITING_MODE` | `1` | Previene bloqueos del pipeline OpenGL/GLX con drivers privativos de NVIDIA. |
| `__NV_DISABLE_EXPLICIT_SYNC` | `1` | Corrige el fallo `Wayland Protocol Error 71` en compositores KWin (KDE Plasma 6) y Mutter (GNOME 46+). |

*Nota: Si la variable de entorno ya fue configurada manualmente por el usuario en su sistema, `canaima-gpu` respetará su valor y no la sobreescribirá.*

---

## 🧪 Pruebas Unitarias

Para ejecutar la suite de pruebas:

```bash
cargo test
```

Para verificar cumplimiento estricto con Clippy:

```bash
cargo clippy -- -D warnings
```

---

## 👥 Equipo y Créditos

Desarrollado con dedicación por el **Equipo de Desarrollo de Canaima GNU/Linux**.

- **Sitio Oficial**: [https://canaima.softwarelibre.gob.ve/](https://canaima.softwarelibre.gob.ve/)
- **Repositorio**: [GitHub Sistema Operativo Canaima](https://github.com/SistemaOperativoCanaima)

---

## 📄 Licencia

Este proyecto se distribuye bajo la licencia **GNU General Public License v3.0 (GPL-3.0-only)**. Consulta el archivo `LICENSE` para más detalles.
