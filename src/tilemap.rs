use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{thread_rng, Rng};

#[derive(Resource)]
struct Seed(u32);

#[derive(Resource)]
struct Zoom(f64);

#[derive(Resource)]
struct XShift(f64);

#[derive(Resource)]
struct YShift(f64);

pub struct TileMapPlugin;

// Adiabatic Lapse Rates for dry and wet air [C/m]
const DRY_ADB_LAPSE_RATE: f64 = 9.8/1000.0;
const WET_ADB_LAPSE_RATE: f64 = 5.0/1000.0;

// Grass biome colors
const BOREAL_FOREST_COLOR: Color       = Color::rgb(26.0/255.0, 101.0/255.0, 49.0/255.0);
const GRASSLAND_COLOR: Color           = Color::rgb(157.0/255.0, 183.0/255.0, 92.0/255.0);
const RAINFOREST_COLOR: Color          = Color::rgb(0.0/255.0, 101.0/255.0, 14.0/255.0);
const FOREST_COLOR: Color              = Color::rgb(56.0/255.0, 115.0/255.0, 41.0/255.0);
const WOODLAND_COLOR: Color            = Color::rgb(97.0/255.0, 119.0/255.0, 44.0/255.0);
const TROPICAL_RAINFOREST_COLOR: Color = Color::rgb(0.0/255.0, 133.0/255.0, 19.0/255.0);
const SAVANNA_COLOR: Color             = Color::rgb(154.0/255.0, 180.0/255.0, 54.0/255.0);
const DEAD_COLOR: Color                = Color::rgb(140.0/255.0, 126.0/255.0, 78.0/255.0);

struct MapGen {
    map_size: u32,
    tile_size: f64, 
    tile_scale: f64,
    height_noise: NoiseMap,
    temperature_noise: NoiseMap,
    humidity_noise: NoiseMap,
    zoom: f64,
    x_shift: f64,
    y_shift: f64,
}

struct NoiseMap {
    noise_map: Perlin,
    octaves: u32,
    scale: f64,
    persistance: f64,
    lacunarity: f64,
}

enum Biome {
    Ocean,
    Tundra,
    BorealForest,
    Grassland,
    Rainforest,
    Forest,
    Woodland,
    TropicalRainforest,
    Savanna,
    Desert
}

struct Tile {
    index: usize,
    color: Color,
}

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Seed(829201))
            .insert_resource(Zoom(1.0))
            .insert_resource(XShift(0.0))
            .insert_resource(YShift(0.0))
            .add_startup_system(spawn_map)
            .add_system(update_inputs)
            .add_system(update_map);
    }
}

impl MapGen {
    fn new(seed: u32, zoom: f64, x_shift: f64, y_shift: f64, map_size: u32, tile_size: f64, tile_scale: f64) -> Self {

        let height_noise = NoiseMap {
            noise_map: Perlin::new(seed),
            octaves: 24,
            scale: 100.0 * zoom,
            persistance: 0.3,
            lacunarity: 4.7,
        };

        let temperature_noise = NoiseMap {
            noise_map: Perlin::new(seed),
            octaves: 24,
            scale: 70.0 * zoom,
            persistance: 0.2,
            lacunarity: 4.1,
        };

        let humidity_noise = NoiseMap {
            noise_map: Perlin::new(seed),
            octaves: 8,
            scale: 90.0 * zoom,
            persistance: 0.08,
            lacunarity: 1.2,
        };

        Self {
            map_size,
            tile_size, 
            tile_scale,
            height_noise,
            temperature_noise,
            humidity_noise,
            zoom,
            x_shift,
            y_shift,
        }
    }

    fn pick_biome(&self, height: f64, temperature: f64, percipitation: f64) -> Biome {

        if height <= 0.0 {
            return Biome::Ocean;
        }

        if temperature < 0.0 {
            return Biome::Tundra;
        } else if temperature < 10.0 && temperature >= 0.0 && percipitation >= 25.0 {
            return Biome::BorealForest;
        } else if temperature < 20.0 && temperature >= 0.0 && percipitation < 25.0 {
            return Biome::Grassland;
        } else if temperature < 20.0 && temperature >= 10.0 {
            if percipitation >= 25.0 && percipitation < 100.0 {
                return Biome::Woodland;
            } else if percipitation >= 100.0 && percipitation < 200.0 {
                return Biome::Forest;
            } else {
                return Biome::Rainforest;
            }
        } else if temperature < 20.0 {
            if percipitation >= 100.0 && percipitation < 250.0 {
                return Biome::Savanna;
            } else {
                return Biome::TropicalRainforest;
            }
        } else {
            return Biome::Desert;
        }

    }

    fn pick_tile(&self, biome: Biome) -> Tile {
        match biome {
            Biome::Ocean => Biome::ocean_tile(),
            Biome::Tundra => Biome::tundra_tile(),
            Biome::BorealForest => Biome::boreal_forest_tile(),
            Biome::Grassland => Biome::grassland_tile(),
            Biome::Rainforest => Biome::rainforest_tile(),
            Biome::Forest => Biome::forest_tile(),
            Biome::Woodland => Biome::woodland_tile(),
            Biome::TropicalRainforest => Biome::tropical_rainforest_tile(),
            Biome::Savanna => Biome::savanna_tile(),
            Biome::Desert => Biome::desert_tile(),
        }
    }

    fn get_heights(&self, r_dis: f64, x: f64, y: f64) -> (f64, f64) {
        let globe_noise = self.height_noise.get_value(x, y) * (1.0 - (r_dis + 0.3 + 0.4 * self.height_noise.get_value(-x, -y)));
        let height = 6000.0 * globe_noise - 1000.0;

        let mut absl_height = height;
        if absl_height < 0.0 {
            absl_height = 0.0;
        } 

        (height, absl_height)
    }

    fn get_partial_temp(&self, absl_height: f64, y_dis: f64, lapse_rate: f64, x: f64, y: f64) -> f64 {
        let noisy_temp = 20.0 * self.temperature_noise.get_value(x, y) - 5.0;
        let temperature = -70.0 * y_dis + noisy_temp - (lapse_rate * absl_height);

        temperature
    }

    fn get_percip_temp(&self, absl_height: f64, y_dis: f64, partial_temp: f64, x: f64, y: f64) -> (f64, f64) {
        let mut temp_clamp = partial_temp;
        if temp_clamp < 0.0 {
            temp_clamp = 0.0;
        }
        if temp_clamp > 20.0 {
            temp_clamp = 0.0;
        }

        let mut evap_prob = 1.0 - ((temp_clamp - 10.0) / 10.0).abs();
        if evap_prob < 0.0 {
            evap_prob = 0.0;
        }

        let avg_lapse_rate = ((WET_ADB_LAPSE_RATE * evap_prob) + (DRY_ADB_LAPSE_RATE * (1.0 - evap_prob))) / 2.0;

        let true_temp = self.get_partial_temp(absl_height, y_dis, avg_lapse_rate, x, y);
        
        let mut water_map = absl_height;
        if water_map == 0.0 {
            water_map = 1.0;
        } else {
            water_map = 0.0;
            if true_temp > -20.0 && true_temp < 40.0 {

                if absl_height < 2000.0 {
                    water_map += evap_prob * (1.0 - (absl_height / 2000.0));
                }

                if water_map > 0.99 {
                    water_map = 0.99;
                }

            }
        }

        let humidity = (0.90 * water_map) + (0.10 * self.humidity_noise.get_value(x, y));

        // let mut percipitation_potential = 19.3 * true_temp + 193.0;
        // if true_temp > 30.0 {
        //     percipitation_potential = -70.0 * true_temp + 2872.0;
        // }

        let percipitation = 400.0 * humidity;

        (percipitation, true_temp)
    }

    fn get_tile(&self, x: f64, y: f64) -> Tile {

        let map_axis_len = self.tile_size * self.tile_scale * self.map_size as f64 / 2.0;

        let x = (x / self.zoom) + (map_axis_len * self.zoom * self.x_shift);
        let y = (y / self.zoom) + (map_axis_len * self.zoom * self.y_shift);

        let y_dis = y / map_axis_len / self.zoom;
        let x_dis = x / map_axis_len / self.zoom;
        let r_dis = ((y_dis * y_dis) + (x_dis * x_dis)).sqrt() / (2.0_f64).sqrt();

        let (height, absl_height) = self.get_heights(r_dis, x, y);
        let partial_temp = self.get_partial_temp(absl_height, y_dis, DRY_ADB_LAPSE_RATE * 0.5, x, y);
        let (percipitation, temperature) = self.get_percip_temp(absl_height, y_dis, partial_temp, x, y);

        let height_clamp = height as f32 / 6000.0;
        let temp_clamp = (temperature as f32 + 33.0) / 88.0;
        let humidity_clamp = percipitation as f32 / 750.0;

        let height_color = Color::rgb(height_clamp, height_clamp, height_clamp);
        let temperature_color = Color::rgb(temp_clamp, temp_clamp, temp_clamp);
        let humidity_color = Color::rgb(humidity_clamp, humidity_clamp, humidity_clamp);

        let mix_color  = Color::rgb(height_clamp, temp_clamp, humidity_clamp);

        let biome: Biome = self.pick_biome(height, temperature, percipitation);
        //let biome = Biome::BorealForest;

        // Rainforest,
        // Forest,
        // Woodland,
        // TropicalRainforest,
        // Savanna,

        let tile = self.pick_tile(biome);

        tile

        // Tile {
        //     index: 0,
        //     color: humidity_color,
        // }

    }
}

impl NoiseMap {
    fn get_value(&self, x: f64, y: f64) -> f64 {
        let mut amplitude: f64 = 1.0;
        let mut frequency: f64 = 1.0;
        let mut accumulation: f64 = 0.0;

        for i in 0..self.octaves {
            let sample_x = x / self.scale * frequency;
            let sample_y = y / self.scale * frequency;

            let value = (self.noise_map.get([sample_x, sample_y]) + 1.0) / 2.0;
            accumulation += value * amplitude;

            amplitude *= self.persistance;
            frequency *= self.lacunarity;
        }

        if accumulation > 1.0 {
            accumulation = 1.0
        }

        if accumulation < 0.0 {
            accumulation = 0.0
        }

        accumulation
    }
}

impl Biome {
    fn ocean_tile() -> Tile {
        Tile {
            index: 3,
            color: Color::rgb(0.0, 0.2, 0.8),
        }
    }

    fn tundra_tile() -> Tile {
        Tile {
            index: 3,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }

    fn boreal_forest_tile() -> Tile {
        Tile {
            index: 0,
            color: BOREAL_FOREST_COLOR,
        }
    }

    fn grassland_tile() -> Tile {
        Tile {
            index: 0,
            color: GRASSLAND_COLOR,
        }
    }

    fn rainforest_tile() -> Tile {
        Tile {
            index: 0,
            color: RAINFOREST_COLOR,
        }
    }

    fn forest_tile() -> Tile {
        Tile {
            index: 0,
            color: FOREST_COLOR,
        }
    }

    fn woodland_tile() -> Tile {
        Tile {
            index: 0,
            color: WOODLAND_COLOR,
        }
    }

    fn tropical_rainforest_tile() -> Tile {
        Tile {
            index: 0,
            color: TROPICAL_RAINFOREST_COLOR,
        }
    }

    fn savanna_tile() -> Tile {
        Tile {
            index: 0,
            color: SAVANNA_COLOR,
        }
    }

    fn desert_tile() -> Tile {
        Tile {
            index: 2,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }
}

fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    seed: Res<Seed>,
    zoom: Res<Zoom>,
    x_shift: Res<XShift>,
    y_shift: Res<YShift>,
) {

    let tile_size = 16.0;
    let tile_scale = 0.25;
    let map_size = 250;

    let mapgen = MapGen::new(seed.0, zoom.0, x_shift.0, y_shift.0, map_size, tile_size, tile_scale);

    let texture_handle = asset_server.load("textures/tilemap.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(tile_size as f32, tile_size as f32), 6, 1, None, None);
    commands.spawn(Camera2dBundle::default());
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let map_half_size: i32 = map_size as i32 / 2;

    for y in -map_half_size..(map_half_size + 1) {
        for x in -map_half_size..(map_half_size + 1) {
            
            let tile_x = x as f64 * tile_size * tile_scale;
            let tile_y = y as f64 * tile_size * tile_scale;

            let tile = mapgen.get_tile(tile_x, tile_y);

            let mut sprite = TextureAtlasSprite::new(tile.index);
            sprite.color = tile.color;

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    sprite: sprite,
                    transform: Transform {
                        translation: Vec3::new(tile_x as f32, tile_y as f32, 0.0),
                        scale: Vec3::splat(tile_scale as f32),
                        ..Default::default()
                    },
                    ..default()
                },
            ));
        }
    }
}

fn update_inputs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    keyboard: Res<Input<KeyCode>>,
    query: Query<Entity>,
    mut seed: ResMut<Seed>,
    mut zoom: ResMut<Zoom>,
    mut x_shift: ResMut<XShift>,
    mut y_shift: ResMut<YShift>,
) {

    if keyboard.just_pressed(KeyCode::Space) {
        let mut rng = thread_rng();

        seed.0 = rng.gen_range(0..99999);
    }
    
    if keyboard.just_pressed(KeyCode::Up) {
        zoom.0 += 0.1;
    }

    if keyboard.just_pressed(KeyCode::Down) && zoom.0 > 0.2 {
        zoom.0 -= 0.1;
    }

    if keyboard.just_pressed(KeyCode::A) && x_shift.0 > -0.9 {
        x_shift.0 -= 0.1 / zoom.0;
    }

    if keyboard.just_pressed(KeyCode::D) && x_shift.0 < 0.9 {
        x_shift.0 += 0.1 / zoom.0;
    }

    if keyboard.just_pressed(KeyCode::S) && y_shift.0 > -0.9 {
        y_shift.0 -= 0.1 / zoom.0;
    }

    if keyboard.just_pressed(KeyCode::W) && y_shift.0 < 0.9 {
        y_shift.0 += 0.1 / zoom.0;
    }
}

fn update_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    keyboard: Res<Input<KeyCode>>,
    query: Query<Entity>,
    seed: Res<Seed>,
    zoom: Res<Zoom>,
    x_shift: Res<XShift>,
    y_shift: Res<YShift>,

) {
    if seed.is_changed() || zoom.is_changed() || x_shift.is_changed() || y_shift.is_changed() {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    
        spawn_map(commands, asset_server, texture_atlases, seed.into(), zoom.into(), x_shift.into(), y_shift.into());
        //println!("{}", zoom.0);
    }
}