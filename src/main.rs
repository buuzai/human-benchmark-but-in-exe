#![windows_subsystem = "windows"]

use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use windows::core::{w, Interface, Result, BOOL, HSTRING, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Input::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_numerics::Vector2;

const RED: D2D1_COLOR_F = rgb(0xce2636);
const GREEN: D2D1_COLOR_F = rgb(0x4bdb6a);
const BLUE: D2D1_COLOR_F = rgb(0x2b87d1);
const WHITE: D2D1_COLOR_F = rgb(0xffffff);

const INTRO: &str = "When the red screen turns green, click as quickly as you can.\nClick anywhere to start.";

const PENALTY_MS: f64 = 350.0;
const MIN_DELAY_US: u64 = 2_000_000;
const DELAY_SPAN_US: u64 = 3_000_001;
const HOT_LIMIT: Duration = Duration::from_secs(5);
const DEDUPE: Duration = Duration::from_millis(20);
const RI_MOUSE_LEFT_BUTTON_DOWN: u16 = 0x0001;

static RESIZED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "trace")]
fn trace_write(line: &str) {
    use std::io::Write;
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("rt_trace.log") {
        let _ = writeln!(f, "{t} {line}");
    }
}

macro_rules! trace {
    ($($a:tt)*) => {
        #[cfg(feature = "trace")]
        trace_write(&format!($($a)*));
    };
}

const fn rgb(hex: u32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Clock,
    Warn,
    Dots,
}

struct Screen {
    color: D2D1_COLOR_F,
    icon: Icon,
    title: String,
    sub: String,
    foot: bool,
}

struct Gfx {
    ctx: ID3D11DeviceContext,
    swap: IDXGISwapChain1,
    d2d: ID2D1Factory,
    dw: IDWriteFactory,
    stroke: ID2D1StrokeStyle,
    rt: Option<ID2D1RenderTarget>,
    brush: Option<ID2D1SolidColorBrush>,
    fonts: Option<[IDWriteTextFormat; 3]>,
    w: f32,
    h: f32,
    tearing: bool,
    swap_flags: DXGI_SWAP_CHAIN_FLAG,
    present_flags: DXGI_PRESENT,
}

impl Gfx {
    unsafe fn new(hwnd: HWND) -> Result<Self> {
        let mut device = None;
        let mut ctx = None;
        let mut made = Ok(());
        for driver in [D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP] {
            made = D3D11CreateDevice(
                None,
                driver,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut ctx),
            );
            if made.is_ok() {
                break;
            }
        }
        made?;
        let device: ID3D11Device = device.unwrap();
        let ctx: ID3D11DeviceContext = ctx.unwrap();

        let dxgi_device: IDXGIDevice1 = device.cast()?;
        let _ = dxgi_device.SetMaximumFrameLatency(1);
        let factory: IDXGIFactory2 = dxgi_device.GetAdapter()?.GetParent()?;

        let mut allow = BOOL(0);
        let tearing_ok = factory
            .cast::<IDXGIFactory5>()
            .map(|f| {
                f.CheckFeatureSupport(
                    DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                    &mut allow as *mut _ as *mut c_void,
                    size_of::<BOOL>() as u32,
                )
                .is_ok()
                    && allow.as_bool()
            })
            .unwrap_or(false);

        let attempts = [
            (DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_SCALING_NONE, 2, tearing_ok),
            (DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_SCALING_NONE, 2, false),
            (DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_SCALING_NONE, 2, false),
            (DXGI_SWAP_EFFECT_DISCARD, DXGI_SCALING_STRETCH, 1, false),
        ];
        let mut swap = None;
        let mut tearing = false;
        let mut last_err = None;
        for (effect, scaling, buffers, tear) in attempts {
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: buffers,
                Scaling: scaling,
                SwapEffect: effect,
                AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                Flags: if tear { DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0 as u32 } else { 0 },
                ..Default::default()
            };
            match factory.CreateSwapChainForHwnd(&device, hwnd, &desc, None, None) {
                Ok(s) => {
                    swap = Some(s);
                    tearing = tear;
                    break;
                }
                Err(e) => last_err = Some(e),
            }
        }
        let swap = match swap {
            Some(s) => s,
            None => return Err(last_err.unwrap()),
        };
        let _ = factory.MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER);

        let d2d: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let round = D2D1_STROKE_STYLE_PROPERTIES {
            startCap: D2D1_CAP_STYLE_ROUND,
            endCap: D2D1_CAP_STYLE_ROUND,
            dashCap: D2D1_CAP_STYLE_ROUND,
            lineJoin: D2D1_LINE_JOIN_ROUND,
            miterLimit: 10.0,
            ..Default::default()
        };
        let mut gfx = Gfx {
            ctx,
            swap,
            stroke: d2d.CreateStrokeStyle(&round, None)?,
            d2d,
            dw: DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?,
            rt: None,
            brush: None,
            fonts: None,
            w: 1.0,
            h: 1.0,
            tearing,
            swap_flags: if tearing { DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING } else { DXGI_SWAP_CHAIN_FLAG(0) },
            present_flags: if tearing { DXGI_PRESENT_ALLOW_TEARING } else { DXGI_PRESENT(0) },
        };
        gfx.build_target()?;
        Ok(gfx)
    }

    unsafe fn font(&self, size: f32, bold: bool, para: DWRITE_PARAGRAPH_ALIGNMENT) -> Result<IDWriteTextFormat> {
        let weight = if bold { DWRITE_FONT_WEIGHT_BOLD } else { DWRITE_FONT_WEIGHT_NORMAL };
        let f = self.dw.CreateTextFormat(
            w!("Segoe UI"),
            None,
            weight,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            size,
            w!("en-us"),
        )?;
        f.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
        f.SetParagraphAlignment(para)?;
        Ok(f)
    }

    unsafe fn build_target(&mut self) -> Result<()> {
        let surface: IDXGISurface = self.swap.GetBuffer(0)?;
        let sd = surface.GetDesc()?;
        self.w = sd.Width as f32;
        self.h = sd.Height as f32;

        let props = D2D1_RENDER_TARGET_PROPERTIES {
            pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_UNKNOWN, alphaMode: D2D1_ALPHA_MODE_IGNORE },
            dpiX: 96.0,
            dpiY: 96.0,
            ..Default::default()
        };
        let rt = self.d2d.CreateDxgiSurfaceRenderTarget(&surface, &props)?;
        rt.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        self.brush = Some(rt.CreateSolidColorBrush(&WHITE, None)?);
        self.rt = Some(rt);

        let h = self.h;
        self.fonts = Some([
            self.font(h * 0.090, true, DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?,
            self.font(h * 0.032, false, DWRITE_PARAGRAPH_ALIGNMENT_NEAR)?,
            self.font(h * 0.018, false, DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?,
        ]);
        Ok(())
    }

    unsafe fn resize(&mut self) {
        self.rt = None;
        self.brush = None;
        self.fonts = None;
        self.ctx.ClearState();
        self.ctx.Flush();
        let _ = self.swap.ResizeBuffers(0, 0, 0, DXGI_FORMAT_UNKNOWN, self.swap_flags);
        let _ = self.build_target();
    }

    unsafe fn draw(&self, s: &Screen, footer: &str) {
        let (Some(rt), Some(brush), Some(fonts)) = (&self.rt, &self.brush, &self.fonts) else {
            return;
        };
        rt.BeginDraw();
        rt.Clear(Some(&s.color));
        self.icon(rt, brush, s.icon);
        let rows = [
            (s.title.as_str(), 0.36, 0.50),
            (s.sub.as_str(), 0.52, 0.92),
            (if s.foot { footer } else { "" }, 0.94, 0.99),
        ];
        for (i, (text, top, bottom)) in rows.into_iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let wide: Vec<u16> = text.encode_utf16().collect();
            let rect = D2D_RECT_F { left: 0.0, top: self.h * top, right: self.w, bottom: self.h * bottom };
            rt.DrawText(&wide, &fonts[i], &rect, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
        let _ = rt.EndDraw(None, None);
    }

    unsafe fn icon(&self, rt: &ID2D1RenderTarget, brush: &ID2D1SolidColorBrush, icon: Icon) {
        let u = self.h * 0.01;
        let (cx, cy) = (self.w * 0.5, self.h * 0.295);
        let sw = 0.7 * u;
        let at = |x: f32, y: f32| Vector2 { X: cx + x * u, Y: cy + y * u };
        let line = |a: Vector2, b: Vector2| rt.DrawLine(a, b, brush, sw, &self.stroke);
        let dot = |c: Vector2, r: f32| rt.FillEllipse(&D2D1_ELLIPSE { point: c, radiusX: r, radiusY: r }, brush);
        match icon {
            Icon::Dots => {
                for x in [-5.5, 0.0, 5.5] {
                    dot(at(x, 0.0), 1.6 * u);
                }
            }
            Icon::Clock => {
                let ring = D2D1_ELLIPSE { point: at(0.0, 0.8), radiusX: 3.4 * u, radiusY: 3.4 * u };
                rt.DrawEllipse(&ring, brush, sw, &self.stroke);
                line(at(0.0, 0.8), at(1.5, -0.9));
                line(at(0.0, -2.6), at(0.0, -3.7));
                line(at(-1.0, -3.9), at(1.0, -3.9));
            }
            Icon::Warn => {
                let (top, left, right) = (at(0.0, -3.6), at(-4.4, 3.6), at(4.4, 3.6));
                line(top, left);
                line(left, right);
                line(right, top);
                line(at(0.0, -0.9), at(0.0, 1.2));
                dot(at(0.0, 2.4), 0.45 * u);
            }
        }
    }

    #[inline(always)]
    unsafe fn present(&self) {
        let _ = self.swap.Present(0, self.present_flags);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Waiting,
    Ready,
    Result,
    TooSoon,
}

struct App {
    hwnd: HWND,
    gfx: Gfx,
    state: State,
    screen: Screen,
    footer: String,
    raw_ok: bool,
    raw_seen: bool,
    legacy_click: Option<Instant>,
    bounds: RECT,
    t0: Instant,
    deadline: Instant,
    count: u32,
    total: f64,
    best: f64,
    rng: u64,
}

impl App {
    unsafe fn refresh_info(&mut self) {
        let _ = GetWindowRect(self.hwnd, &mut self.bounds);
        let mut hz = 0;
        let mut mi = MONITORINFOEXW::default();
        mi.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        let mon = MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST);
        if GetMonitorInfoW(mon, &mut mi as *mut _ as *mut MONITORINFO).as_bool() {
            let mut dm = DEVMODEW { dmSize: size_of::<DEVMODEW>() as u16, ..Default::default() };
            if EnumDisplaySettingsW(PCWSTR(mi.szDevice.as_ptr()), ENUM_CURRENT_SETTINGS, &mut dm).as_bool() {
                hz = dm.dmDisplayFrequency;
            }
        }
        self.footer = format!(
            "{}{}   |   {}   |   R: reset   |   Esc: quit",
            if hz > 1 { format!("{hz} Hz   |   ") } else { String::new() },
            if self.gfx.tearing { "tearing on" } else { "tearing off" },
            if self.raw_ok { "raw input" } else { "standard input" },
        );
    }

    unsafe fn show(&mut self, color: D2D1_COLOR_F, icon: Icon, title: String, sub: String, foot: bool) {
        self.screen = Screen { color, icon, title, sub, foot };
        self.gfx.draw(&self.screen, &self.footer);
        self.gfx.present();
    }

    fn record(&mut self, ms: f64) {
        self.count += 1;
        self.total += ms;
        if self.count == 1 || ms < self.best {
            self.best = ms;
        }
    }

    fn stats(&self) -> String {
        format!(
            "Attempts: {}   Avg: {:.1} ms   Best: {:.0} ms",
            self.count,
            self.total / self.count as f64,
            self.best
        )
    }

    unsafe fn reset(&mut self) {
        self.count = 0;
        self.total = 0.0;
        self.best = 0.0;
        self.state = State::Idle;
        self.show(BLUE, Icon::Clock, "Reaction Time Test".into(), INTRO.into(), true);
    }

    fn next_delay(&mut self) -> Duration {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        Duration::from_micros(MIN_DELAY_US + x % DELAY_SPAN_US)
    }

    unsafe fn start_wait(&mut self) {
        self.state = State::Waiting;
        self.show(RED, Icon::Dots, "Wait for green".into(), String::new(), false);
        self.screen = Screen { color: GREEN, icon: Icon::Dots, title: "Click!".into(), sub: String::new(), foot: false };
        self.gfx.draw(&self.screen, "");
        self.gfx.ctx.Flush();
        let delay = self.next_delay();
        self.deadline = Instant::now() + delay;
    }

    #[inline(always)]
    unsafe fn go_green(&mut self) {
        self.gfx.present();
        self.t0 = Instant::now();
        self.state = State::Ready;
        trace!("green");
    }

    unsafe fn click(&mut self, now: Instant) {
        match self.state {
            State::Ready => {
                let ms = now.duration_since(self.t0).as_secs_f64() * 1000.0;
                self.record(ms);
                self.state = State::Result;
                trace!("result ms={ms:.3}");
                let sub = format!("Click to keep going.\n{}", self.stats());
                self.show(BLUE, Icon::Clock, format!("{ms:.0} ms"), sub, true);
            }
            State::Waiting => {
                self.record(PENALTY_MS);
                self.state = State::TooSoon;
                trace!("toosoon");
                let sub = format!("+{PENALTY_MS:.0} ms penalty\nClick to try again.\n{}", self.stats());
                self.show(BLUE, Icon::Warn, "Too soon!".into(), sub, true);
            }
            _ => self.start_wait(),
        }
    }

    unsafe fn cursor_inside(&self) -> bool {
        let mut p = POINT::default();
        if GetCursorPos(&mut p).is_err() {
            return true;
        }
        let b = &self.bounds;
        p.x >= b.left && p.x < b.right && p.y >= b.top && p.y < b.bottom
    }

    unsafe fn handle(&mut self, msg: &MSG, now: Instant) -> bool {
        match msg.message {
            WM_QUIT => return false,
            WM_INPUT => {
                let mut raw = RAWINPUT::default();
                let mut size = size_of::<RAWINPUT>() as u32;
                let got = GetRawInputData(
                    HRAWINPUT(msg.lParam.0 as *mut c_void),
                    RID_INPUT,
                    Some(&mut raw as *mut _ as *mut c_void),
                    &mut size,
                    size_of::<RAWINPUTHEADER>() as u32,
                );
                if got != u32::MAX && raw.header.dwType == RIM_TYPEMOUSE.0 {
                    self.raw_seen = true;
                    let down = raw.data.mouse.Anonymous.Anonymous.usButtonFlags & RI_MOUSE_LEFT_BUTTON_DOWN != 0;
                    let dup = self.legacy_click.is_some_and(|t| now.duration_since(t) < DEDUPE);
                    if down && !dup && self.cursor_inside() {
                        trace!("click raw");
                        self.click(now);
                    }
                }
            }
            WM_LBUTTONDOWN => {
                if !self.raw_seen {
                    self.legacy_click = Some(now);
                    trace!("click legacy");
                    self.click(now);
                }
            }
            WM_KEYDOWN => {
                let key = msg.wParam.0 as u16;
                if key == VK_ESCAPE.0 {
                    let _ = DestroyWindow(self.hwnd);
                } else if key == b'R' as u16 {
                    self.reset();
                }
            }
            _ => {}
        }
        DispatchMessageW(msg);
        true
    }

    unsafe fn resized(&mut self) {
        self.gfx.resize();
        self.refresh_info();
        if self.state == State::Waiting {
            self.start_wait();
        } else {
            self.gfx.draw(&self.screen, &self.footer);
            self.gfx.present();
        }
    }
}

unsafe fn monitor_rect(mon: HMONITOR) -> RECT {
    let mut mi = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    if GetMonitorInfoW(mon, &mut mi).as_bool() {
        mi.rcMonitor
    } else {
        RECT { left: 0, top: 0, right: GetSystemMetrics(SM_CXSCREEN), bottom: GetSystemMetrics(SM_CYSCREEN) }
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_SIZE => {
                if wp.0 as u32 != SIZE_MINIMIZED {
                    RESIZED.store(true, Ordering::Relaxed);
                }
                LRESULT(0)
            }
            WM_DISPLAYCHANGE => {
                let rc = monitor_rect(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST));
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    rc.left,
                    rc.top,
                    rc.right - rc.left,
                    rc.bottom - rc.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                RESIZED.store(true, Ordering::Relaxed);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

unsafe fn run() -> Result<()> {
    let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    let _ = SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS);
    let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);

    let hinst: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("ReactionTimeWindow");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: hinst,
        hCursor: LoadCursorW(None, IDC_HAND)?,
        lpszClassName: class,
        ..Default::default()
    };
    RegisterClassW(&wc);

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let rc = monitor_rect(MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY));
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class,
        w!("Reaction Time"),
        WS_POPUP,
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        None,
        None,
        Some(hinst),
        None,
    )?;
    let mouse = RAWINPUTDEVICE { usUsagePage: 1, usUsage: 2, dwFlags: RAWINPUTDEVICE_FLAGS(0), hwndTarget: hwnd };
    let raw_ok = RegisterRawInputDevices(&[mouse], size_of::<RAWINPUTDEVICE>() as u32).is_ok();

    let now = Instant::now();
    let mut seed = 0i64;
    let _ = windows::Win32::System::Performance::QueryPerformanceCounter(&mut seed);
    let mut app = App {
        hwnd,
        gfx: Gfx::new(hwnd)?,
        state: State::Idle,
        screen: Screen { color: BLUE, icon: Icon::Clock, title: String::new(), sub: String::new(), foot: true },
        footer: String::new(),
        raw_ok,
        raw_seen: false,
        legacy_click: None,
        bounds: rc,
        t0: now,
        deadline: now,
        count: 0,
        total: 0.0,
        best: 0.0,
        rng: (seed as u64 ^ 0x9E37_79B9_7F4A_7C15) | 1,
    };
    app.refresh_info();
    app.reset();
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetForegroundWindow(hwnd);
    RESIZED.store(false, Ordering::Relaxed);
    app.gfx.draw(&app.screen, &app.footer);
    app.gfx.present();

    let mut msg = MSG::default();
    'main: loop {
        let hot = match app.state {
            State::Waiting => true,
            State::Ready => app.t0.elapsed() < HOT_LIMIT,
            _ => false,
        };
        if hot {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let now = Instant::now();
                if !app.handle(&msg, now) {
                    break 'main;
                }
            }
            if app.state == State::Waiting && Instant::now() >= app.deadline {
                app.go_green();
            }
            std::hint::spin_loop();
        } else {
            if GetMessageW(&mut msg, None, 0, 0).0 <= 0 {
                break;
            }
            let now = Instant::now();
            if !app.handle(&msg, now) {
                break;
            }
        }
        if RESIZED.swap(false, Ordering::Relaxed) {
            app.resized();
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = unsafe { run() } {
        let text = HSTRING::from(format!("Could not start: {e}"));
        unsafe {
            MessageBoxW(None, &text, w!("Reaction Time"), MB_ICONERROR | MB_OK);
        }
    }
}
