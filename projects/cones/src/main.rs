//! Displays spheres with physically based materials.
use amethyst::{
    assets::AssetLoaderSystemData,
    ecs::{Join, Read, ReadStorage, System, WriteStorage, Component, DenseVecStorage},
    core::{
        ecs::{Builder, WorldExt},
        Transform, TransformBundle,
        timing::Time,
    },
    renderer::{
        camera::Camera,
        light::{Light, PointLight},
        mtl::{Material, MaterialDefaults},
        palette::{LinSrgba, Srgb},
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        shape::Shape,
        types::DefaultBackend,
        Mesh, RenderingBundle, Texture,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, GameDataBuilder, SimpleState, StateData,
};
use nalgebra::Vector3;

pub enum LightColorEnum {
    Red,
    Green,
    None, // hack: probably a better way to do this in Amethyst
}

pub struct LightColor {
    color: LightColorEnum,
}
impl Default for LightColor {
    fn default() -> Self {
        Self {
            color: LightColorEnum::None,
        }
    }
}
impl Component for LightColor {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
struct Example {}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();

        println!("Load mesh");
        let (mesh, albedo) = {
            let mesh = world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load_from_data(
                    Shape::Cone(7)
                        .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                        .into(),
                    (),
                )
            });
            let albedo = world.exec(|loader: AssetLoaderSystemData<'_, Texture>| {
                loader.load_from_data(
                    load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
                    (),
                )
            });

            (mesh, albedo)
        };

        println!("Create shapes");
        let n = 201;
        for i in 0..n {
            for j in 0..n {
                let roughness = 0.0;
                let metallic = 0.0;

                let mut pos = Transform::default();
                pos.set_translation_xyz(2.5f32 * (i - n/2) as f32, 2.5f32 * (j - n/2) as f32, 0.0);
                pos.set_rotation_x_axis(std::f32::consts::PI);

                let mtl = world.exec(
                    |(mtl_loader, tex_loader): (
                        AssetLoaderSystemData<'_, Material>,
                        AssetLoaderSystemData<'_, Texture>,
                    )| {
                        let metallic_roughness = tex_loader.load_from_data(
                            load_from_linear_rgba(LinSrgba::new(0.0, roughness, metallic, 0.0))
                                .into(),
                            (),
                        );

                        mtl_loader.load_from_data(
                            Material {
                                albedo: albedo.clone(),
                                metallic_roughness,
                                ..mat_defaults.clone()
                            },
                            (),
                        )
                    },
                );

                world
                    .create_entity()
                    .with(pos)
                    .with(mesh.clone())
                    .with(mtl)
                    .build();
            }
        }

        println!("Create lights");
        let light1: Light = PointLight {
            intensity: 10.0,
            color: Srgb::new(1.0, 0.0, 0.0),
            ..PointLight::default()
        }
            .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(10.0, 0.0, -3.0);

        let light2: Light = PointLight {
            intensity: 10.0,
            color: Srgb::new(0.0, 1.0, 0.0),
            ..PointLight::default()
        }
            .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(-10.0, 0.0, -3.0);

        world
            .create_entity()
            .with(light1)
            .with(light1_transform)
            .with(LightColor{color: LightColorEnum::Red})
            .build();

        world
            .create_entity()
            .with(light2)
            .with(light2_transform)
            .with(LightColor{color: LightColorEnum::Green })
            .build();

        println!("Put camera");

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -12.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        world
            .create_entity()
            .with(Camera::standard_3d(width, height))
            .with(transform)
            .build();
    }
}

pub struct MoveLightsSystem;

impl<'s> System<'s> for MoveLightsSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Light>,
        Read<'s, Time>,
        ReadStorage<'s, LightColor>
    );

    fn run(&mut self, (mut transforms, lights, time, light_colors): Self::SystemData) {
        for (light_color, transform) in (&light_colors, &mut transforms).join() {
            let seconds = time.absolute_real_time_seconds() as f32;
            let movement_y = -(seconds*10.0).sin()*100.0;
            let movement_x = (seconds*10.0).cos()*100.0;
            match light_color.color {
                LightColorEnum::Red => transform.set_translation_xyz(movement_x, movement_y, -3.0),
                LightColorEnum::Green => transform.set_translation_xyz(movement_y, movement_x, -3.0),
                _ => transform,
            };
        }
    }
}

pub struct MoveCameraSystem;

impl<'s> System<'s> for MoveCameraSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, camera, time): Self::SystemData) {
        for (cam, transform) in (&camera, &mut transforms).join() {
            let seconds = time.absolute_real_time_seconds() as f32;
            transform.set_translation_xyz(-8.0*seconds.sin(), 8.0*seconds.cos(), -5.0);
            transform.face_towards(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderPbr3D::default()),
        )?
        .with(MoveLightsSystem, "move_lights_system", &[])
        .with(MoveCameraSystem, "move_camera_system", &[]);

    let mut game = Application::new(assets_dir, Example::default(), game_data)?;
    game.run();
    Ok(())
}
