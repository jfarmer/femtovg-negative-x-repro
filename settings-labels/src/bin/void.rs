use femtovg::{renderer::Void, Canvas, TextContext};
fn main() {
    let ctx = TextContext::default();
    let mut canvas = Canvas::new_with_text_context(Void, ctx.clone()).unwrap();
    canvas.set_size(900, 900, 1.);
    let scene = settings_labels::Scene::new(&ctx);
    settings_labels::measure(&mut canvas, &scene, |c| c.flush_to_output(()), |_| {});
}
