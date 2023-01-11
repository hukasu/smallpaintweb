use yew::prelude::*;
use smallpaint::{renderer::Renderer, tracer::{Tracer, PainterlyTracer}, camera::{Camera, SimpleCamera}, Scene, common::DepthTerminator, sampler::HaltonSampler};
pub use wasm_bindgen_rayon::init_thread_pool;

mod scene_builder;

struct RendererObjects {
    renderer: Renderer,
    tracer: Box<dyn Tracer>,
    camera: Box<dyn Camera>,
    scene: Scene
}

enum SmallpaintMessage {
    SamplePerPixelChange(u64),
    RefractionIndexChange(f64),
    CanvasUpdate,
    RendererStart,
    RendererResume,
    RendererPause,
    RendererStop
}

struct SmallpaintComponent {
    samples_per_pixel: u64,
    refraction_index: f64,
    scene: scene_builder::PrebuiltScene,
    renderer: Option<RendererObjects>,
    renderer_interval: Option<gloo_timers::callback::Interval>
}

impl Component for SmallpaintComponent {
    type Message = SmallpaintMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            samples_per_pixel: 10,
            refraction_index: 1.5,
            scene: scene_builder::PrebuiltScene::ThreeSpheres,
            renderer: None,
            renderer_interval: None
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        html!(
            <>
            <div>
                <button onclick={link.callback(|_| Self::Message::RendererStart)}>{"Start"}</button>
                <canvas id="smallpaint-canvas" width="512" height="512">
                </canvas>
            </div>
            </>
        )
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::SamplePerPixelChange(spp) => {
                self.samples_per_pixel = spp;
                false
            },
            Self::Message::RefractionIndexChange(ri) => {
                self.refraction_index = ri;
                false
            },
            Self::Message::CanvasUpdate => {
                gloo::console::debug!("Timer called");
                true
            }
            Self::Message::RendererStart => {
                let old = self.renderer.replace(
                    RendererObjects {
                        renderer: Renderer::new(
                            512,
                            512,
                            self.refraction_index,
                            self.samples_per_pixel
                        ),
                        tracer: Box::new(
                            PainterlyTracer::new(
                                Box::new(DepthTerminator::new(20)),
                                Box::new(HaltonSampler::new())
                            )
                        ),
                        camera: Box::new(SimpleCamera::new(512., 512.)),
                        scene: self.scene.build()
                    }
                );
                
                if let Some(old_rend) = old {
                    old_rend.renderer.stop().unwrap();
                }
                
                let canvas_updater = {
                    let link = ctx.link().clone();
                    gloo_timers::callback::Interval::new(
                        500,
                        move || link.send_message(Self::Message::CanvasUpdate)
                    )
                };
                let old_updater = self.renderer_interval.replace(canvas_updater);
                
                if let Some(old_upd) = old_updater {
                    old_upd.cancel();
                }

                if let Some(rend) = &mut self.renderer {
                    rend.renderer.render(rend.tracer.as_ref(), rend.camera.as_ref(), &rend.scene).unwrap();
                }

                true
            },
            Self::Message::RendererResume => {
                if let Some(rend) = &self.renderer {
                    rend.renderer.resume().unwrap();
                }
                true
            },
            Self::Message::RendererPause => {
                if let Some(rend) = &self.renderer {
                    rend.renderer.pause().unwrap();
                }
                true
            },
            Self::Message::RendererStop => {
                if let Some(rend) = &self.renderer {
                    rend.renderer.stop().unwrap();
                }
                true
            }
        }
    }
}

fn main() {
    yew::Renderer::<SmallpaintComponent>::new().render();
}