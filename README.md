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

**`canaima-gpu`** es una librería en Rust diseñada específicamente para garantizar la máxima estabilidad y rendimiento visual en aplicaciones de escritorio basadas en **Tauri v2** y **WebKitGTK** sobre Canaima GNU/Linux y el ecosistema Debian.

En entornos Linux modernos coexisten múltiples servidores gráficos (Wayland y X11), compositores variados y diversos fabricantes de hardware gráfico (NVIDIA, Intel, AMD). Sin una configuración adecuada, las aplicaciones WebKitGTK suelen presentar fallos graves:
- ❌ **Pantallas blancas o congelamientos repentinos** al iniciar la aplicación.
- ❌ **`Wayland Protocol Error 71`** ocasionado por incompatibilidades en sincronización explícita (`explicit sync`) con controladores NVIDIA.
- ❌ Fallos con el renderizador de búferes de memoria directa (**DMA-BUF**) en compositores Wayland.
- ❌ Degradación injustificada a renderizado por software en CPU en tarjetas **Intel (Arc/Iris/UHD)** o **AMD (Radeon)** por la aplicación indiscriminada de parches globales.

`canaima-gpu` soluciona esto de raíz: **analiza el entorno de hardware y software en menos de 1 milisegundo**, detecta con precisión el entorno de escritorio activo y aplica de manera quirúrgica las variables de entorno necesarias únicamente cuando el hardware lo requiere.

---

## ✨ Características Principales

- ⚡ **Inspección de Hardware en < 1 ms**: No ejecuta llamadas lentas de subprocesos (`lspci`, `nvidia-smi` o `grep`). Consulta directamente los sistemas de archivos del kernel (`/sys/bus/pci/devices`, `/proc/driver/nvidia`, nodos de caracteres `/dev/nvidia*`).
- 🎯 **Detección Multinivel de NVIDIA**:
  1. `/proc/driver/nvidia` (controladores privativos u open del kernel).
  2. Nodos del kernel `/sys/module/nvidia*`.
  3. Dispositivos `/dev/nvidiactl` y `/dev/nvidia0`.
  4. Lista de módulos activos en `/proc/modules`.
  5. Escaneo exhaustivo del bus PCI filtrando por clase de controlador de pantalla (`0x03xxxx`) y Vendor ID oficial (`0x10de`).
- 🖥️ **Soporte Completo para 7 Entornos de Escritorio**: Detecta y se adapta automáticamente a:
  - **GNOME** (Mutter)
  - **KDE Plasma** (KWin)
  - **XFCE** (Xfwm4)
  - **Cinnamon** (Muffin)
  - **MATE** (Marco)
  - **LXQt** (Openbox / KWin)
  - **LXDE** (Openbox)
- 🛡️ **Parches Inteligentes y No Invasivos**: No penaliza a usuarios con tarjetas gráficas Intel ni AMD; la aceleración por hardware *zero-copy* (DMA-BUF) se mantiene activa por defecto.
- 🌐 **Zero Dependencies**: Implementado 100% con la biblioteca estándar de Rust (`std`), sin dependencias pesadas de terceros.

---

## 🖥️ Matriz de Compatibilidad por Entorno

| Entorno de Escritorio | Gestor / Compositor | Sesión | Comportamiento del Parche |
| :--- | :--- | :--- | :--- |
| **GNOME** | Mutter | Wayland & X11 | Aplica `WEBKIT_DISABLE_DMABUF_RENDERER` y `__NV_DISABLE_EXPLICIT_SYNC` si hay NVIDIA. Intel/AMD mantienen aceleración completa. |
| **KDE Plasma** | KWin | Wayland & X11 | Previene el *Protocol Error 71* en KWin Wayland con NVIDIA privativo. |
| **XFCE** | Xfwm4 | X11 | Compatibilidad asegurada en entornos X11 clásicos; mantiene DMA-BUF en Intel/AMD. |
| **Cinnamon** | Muffin | X11 & Wayland | Parches automáticos para compositores derivados de Mutter con NVIDIA. |
| **MATE** | Marco | X11 | Compatible con Marco en modo por software o con composición GL activada. |
| **LXQt** | Openbox / KWin | X11 & Wayland | Aceleración en compositores ligeros y protección en tarjetas dedicadas. |
| **LXDE** | Openbox | X11 | Modo seguro en NVIDIA para evitar fallos de renderizado en escritorios ligeros sin compositor. |

---

## 📦 Instalación

En el archivo `Cargo.toml` de tu backend Tauri (`src-tauri/Cargo.toml`):

```toml
[dependencies]
# Ruta local en el repositorio Canaima
canaima-gpu = { path = "../../../lib-rust-gpu-main" }

# O mediante repositorio Git
# canaima-gpu = { git = "https://github.com/SistemaOperativoCanaima/rust-gpu-canaima.git", branch = "main" }
```

---

## 🛠️ Guía Paso a Paso: ¿Cómo integrarlo en tu App Tauri?

Para que los parches de WebKitGTK tengan efecto, **es obligatorio invocarlos antes de que el motor web se inicialice**. 

### 1. Invocación temprana en `src-tauri/src/main.rs`

El desarrollador **solo debe agregar una línea** al inicio absoluto de la función `main()`:

```rust
// src-tauri/src/main.rs

// Evita la ventana de consola adicional en Windows en modo release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 1. APLICAR PARCHES GRÁFICOS ANTES DE CUALQUIER HILO O RUNTIME
    canaima_gpu::apply_workarounds();

    // 2. Iniciar el runtime de la aplicación Tauri
    mi_app_lib::run();
}
```

> [!IMPORTANT]
> **¿Por qué en `main.rs` y no en `setup()` de Tauri?**
> Las variables de entorno leídas por WebKitGTK (`WEBKIT_DISABLE_DMABUF_RENDERER`, `__NV_DISABLE_EXPLICIT_SYNC`, etc.) son procesadas por el motor web en el momento en que se cargan sus bibliotecas dinámicas. Además, según el estándar POSIX, modificar variables de entorno (`std::env::set_var`) una vez que existen múltiples hilos concurrentes puede generar carreras de memoria. Invocando `canaima_gpu::apply_workarounds()` al inicio de `main()`, se garantiza seguridad total y aplicación efectiva.

---

### 2. Opciones avanzadas de personalización (`GpuOptions`)

Si tu aplicación requiere forzar algún parámetro específico (por ejemplo, para diagnósticos en soporte técnico o depuración), puedes usar `apply_workarounds_with_options`:

```rust
// src-tauri/src/main.rs
use canaima_gpu::{apply_workarounds_with_options, GpuOptions};

fn main() {
    let opciones = GpuOptions {
        // Forzar desactivación de DMA-BUF independientemente del hardware
        force_disable_dmabuf: false,
        // Forzar desactivación del modo de composición acelerada
        force_disable_compositing: false,
        // Forzar desactivación de explicit sync
        force_disable_explicit_sync: false,
        // Imprimir diagnóstico detallado en la terminal al arrancar
        verbose: true,
    };

    apply_workarounds_with_options(&opciones);

    mi_app_lib::run();
}
```

Salida típica en consola con `verbose: true`:
```text
🔧 [canaima-gpu] Diagnóstico -> NVIDIA: false, Wayland: true, Entorno: Gnome
```

---

### 3. Consultas de entorno en tu código de negocio

Puedes usar las funciones de detección de `canaima-gpu` para adaptar la interfaz o la lógica interna de tu aplicación:

```rust
use canaima_gpu::{
    is_nvidia_active, is_wayland, get_desktop_environment, DesktopEnvironment
};

pub fn inspeccionar_sistema() {
    if is_nvidia_active() {
        println!("🎮 GPU NVIDIA activa detectada.");
    }

    if is_wayland() {
        println!("🚀 Sesión sobre Wayland.");
    } else {
        println!("🖥️ Sesión sobre servidor X11.");
    }

    match get_desktop_environment() {
        DesktopEnvironment::Gnome => println!("Entorno de escritorio: GNOME"),
        DesktopEnvironment::KdePlasma => println!("Entorno de escritorio: KDE Plasma"),
        DesktopEnvironment::Xfce => println!("Entorno de escritorio: XFCE"),
        DesktopEnvironment::Cinnamon => println!("Entorno de escritorio: Cinnamon"),
        DesktopEnvironment::Mate => println!("Entorno de escritorio: MATE"),
        DesktopEnvironment::Lxqt => println!("Entorno de escritorio: LXQt"),
        DesktopEnvironment::Lxde => println!("Entorno de escritorio: LXDE"),
        DesktopEnvironment::Other => println!("Otro gestor de ventanas"),
    }
}
```

---

## ⚙️ Variables de Entorno Gestionadas

Cuando se detecta una GPU NVIDIA activa (o se solicita forzadamente vía opciones), `canaima-gpu` configura:

| Variable | Valor | Motivo Técnico |
| :--- | :---: | :--- |
| `WEBKIT_DISABLE_DMABUF_RENDERER` | `1` | Desactiva el paso de búferes DMA problemáticos en controladores privativos NVIDIA sobre compositores Wayland, evitando pantallas blancas. |
| `WEBKIT_DISABLE_COMPOSITING_MODE` | `1` | Previene bloqueos del pipeline OpenGL/GLX con drivers de vídeo en sesiones KDE Wayland con NVIDIA. |
| `__NV_DISABLE_EXPLICIT_SYNC` | `1` | Corrige el fallo crítico `Wayland Protocol Error 71` en compositores KWin y Mutter modernos. |

> [!NOTE]
> Si el usuario o el administrador del sistema ya ha definido previamente alguna de estas variables en su entorno, `canaima-gpu` **respetará su valor preexistente** y no lo sobreescribirá.

---

## 🧪 Pruebas Unitarias y Calidad

Para ejecutar la batería de pruebas automatizadas:

```bash
cargo test
```

Para validar el código bajo las normas estrictas de Rust:

```bash
cargo clippy -- -D warnings
```

---

## 👥 Equipo y Créditos

Desarrollado con dedicación por el **Equipo de Desarrollo de Canaima GNU/Linux**.

- **Sitio Web Oficial**: [https://canaima.softwarelibre.gob.ve/](https://canaima.softwarelibre.gob.ve/)
- **Organización en GitHub**: [Sistema Operativo Canaima](https://github.com/SistemaOperativoCanaima)

---

## 📄 Licencia

Este proyecto está licenciado bajo los términos de la **GNU General Public License v3.0 (GPL-3.0-only)**. Consulta el archivo `LICENSE` para más información.
