use femtovg::{renderer::OpenGl, Canvas, ImageFlags, PixelFormat, RenderTarget};
use std::ffi::{c_char, c_void, CStr};
use std::ptr::{null, null_mut};

#[link(name = "OpenGL", kind = "framework")]
extern "C" {
    fn CGLChoosePixelFormat(attrs: *const u32, format: *mut *mut c_void, count: *mut i32) -> i32;
    fn CGLCreateContext(format: *mut c_void, shared: *mut c_void, context: *mut *mut c_void) -> i32;
    fn CGLSetCurrentContext(context: *mut c_void) -> i32;
    fn glFinish();
    fn glGetString(name: u32) -> *const c_char;
}
extern "C" {
    fn dlsym(handle: *const c_void, name: *const c_char) -> *const c_void;
}
fn main() {
    unsafe {
        let (mut format, mut count, mut context) = (null_mut(), 0, null_mut());
        assert_eq!(
            CGLChoosePixelFormat([99, 0x3200, 73, 0].as_ptr(), &mut format, &mut count),
            0
        );
        assert_eq!(CGLCreateContext(format, null_mut(), &mut context), 0);
        assert_eq!(CGLSetCurrentContext(context), 0);
        for name in [0x1f01, 0x1f02] {
            let info = glGetString(name);
            assert_ne!(info, null());
            eprintln!("{}", CStr::from_ptr(info).to_string_lossy());
        }
        let gl = OpenGl::new_from_function_cstr(|name| dlsym((-2isize) as *const c_void, name.as_ptr())).unwrap();
        let ctx = femtovg::TextContext::default();
        let mut canvas = Canvas::new_with_text_context(gl, ctx.clone()).unwrap();
        canvas.set_size(900, 900, 1.);
        let image = canvas
            .create_image_empty(900, 900, PixelFormat::Rgba8, ImageFlags::empty())
            .unwrap();
        canvas.set_render_target(RenderTarget::Image(image));
        let scene = settings_labels::Scene::new(&ctx);
        settings_labels::measure(&mut canvas, &scene, |c| c.flush(), |_| glFinish());
        if let Some(path) = std::env::args().nth(2) {
            let pixels = canvas.screenshot().unwrap();
            let bytes: Vec<_> = pixels.pixels().flat_map(|p| [p.r, p.g, p.b, p.a]).collect();
            assert!(
                bytes
                    .chunks_exact(4)
                    .map(|p| p[0])
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    >= 20
            );
            std::fs::write(path, bytes).unwrap();
        }
    }
}
