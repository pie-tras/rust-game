use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{thread_rng, Rng};

pub struct TileMapPlugin;

struct MapGen {
    map_size: u32,
    tile_size: f32, 
    tile_scale: f32,
    height_noise: NoiseMap,
    temperature_noise: NoiseMap,
    humidity_noise: NoiseMap,
}

struct NoiseMap {
    noise_map: Perlin,
    octaves: u32,
    scale: f64,
    persistance: f64,
    lacunarity: f64,
}

struct Tile {
    index: usize,
    color: Color,
}

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_map)
            .add_system(respawn_map);
    }
}

impl MapGen {
    fn new(seed: u32, map_size: u32, tile_size: f32, tile_scale: f32) -> Self {

        let height_noise = NoiseMap {
            noise_map: Perlin::new(seed),
            octaves: 8,
            scale: 250.0,
            persistance: 0.07,
            lacunarity: 4.7,
        };

        let temperature_noise = NoiseMap {
            noise_map: Perlin::new(seed),
            octaves: 8,
            scale: 100.0,
            persistance: 0.15,
            lacunarity: 3.2,
        };

        let humidity_noise = NoiseMap {
            noise_map: Perlin::new(seed),
            octaves: 8,
            scale: 100.0,
            persistance: 0.15,
            lacunarity: 3.2,
        };

        Self {
            map_size,
            tile_size, 
            tile_scale,
            height_noise,
            temperature_noise,
            humidity_noise,
        }
    }

    fn get_tile(&self, x: f64, y: f64) -> Tile {

        let max_dis = self.tile_size * self.tile_scale * (self.map_size as f32 / (2.0_f32).sqrt());
        let r_dis = ((x * x) + (y * y)).sqrt();
        let norm_dis = r_dis / max_dis as f64;
        let y_dis = y as f32 / (self.tile_size * self.tile_scale * self.map_size as f32 / 2.0);

        let mut height = 6000.0 * (self.height_noise.get_value(x, y) - 2.5 * (norm_dis.powf(2.5) + 1.0).ln()) - 2000.0;
        let mut temp: f64 = 44.0 * (-y_dis as f64) + 11.0 + 12.0 * self.temperature_noise.get_value(x, y);

        let mut index: usize = 0;
        let mut color = Color::rgb(1.0, 1.0, 1.0);
        
        if height <= 0.0 {
            index = 3;

            let mut cold_temp = temp as f32;
            if cold_temp < 0.0 {
                cold_temp = 0.0;
            }
            if cold_temp > 10.0 {
                cold_temp = 10.0;
            }

            let mut hot_temp = temp as f32;
            if hot_temp < 0.0 {
                hot_temp = 0.0;
            }
            if hot_temp > 40.0 {
                hot_temp = 40.0;
            }

            let cold_temp = 0.05 * (1.0 - (cold_temp / 10.0));
            let hot_temp = 0.1 * (hot_temp / 40.0);

            let water_color = (height as f32 / 4000.0) + 0.3 - cold_temp + hot_temp;            
            color = Color::rgb(0.1, water_color + 0.1, water_color + 0.2);
        } else if height > 0.0 && height <= 10.0 {
            index = 2;
        } else if height > 10.0 && height <= 3000.0 {
            index = 0;
            let b: f32 = 150.0/255.0 * (height as f32 / 3000.0);

            let mut grass_temp = temp as f32;
            if grass_temp < 0.0 {
                grass_temp = 0.0;
            }
            if grass_temp > 29.0 {
                grass_temp = 29.0;
            }

            color = Color::rgb(0.75 * (grass_temp / 29.0), 170.0/255.0, b);
        } else if height > 3000.0 && height <= 3200.0 {
            index = 1;
            //color = Color::rgb(140.0/255.0, 108.0/255.0, 60.0/255.0);
        } else if height > 3200.0 && height <= 3700.0 {
            index = 4;
        } else {
            color = Color::rgb(1.0, 1.0, 1.0);
            index = 3;
        }

        if temp < 0.0 {
            if index == 3 && height > -20.0 {
                color = Color::rgb(1.0, 1.0, 1.0);
                index = 5;
            } else if index == 2 {
                index = 4;
            } else if index == 0 {
                color = Color::rgb(1.0, 1.0, 1.0);
                index = 3;
            }
        }


        if temp >= 30.0 && temp < 38.0 {
            if index == 0 {
                color = Color::rgb(1.0, 1.0, 1.0);
                index = 1;
            }
        }else if temp >= 38.0 {
            if index == 0 {
                color = Color::rgb(1.0, 1.0, 1.0);
                index = 2;
            }
        }

        Tile {
            index,
            color,
        }
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

fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {

    let tile_size = 16.0;
    let tile_scale = 0.5;
    let map_size = 150;

    let mut rng = thread_rng();

    //829201
    let mapgen = MapGen::new(rng.gen_range(0..99999), map_size, tile_size, tile_scale);

    let texture_handle = asset_server.load("textures/tilemap.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(tile_size, tile_size), 6, 1, None, None);
    commands.spawn(Camera2dBundle::default());
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let map_half_size: i32 = map_size as i32 / 2;

    for y in -map_half_size..(map_half_size + 1) {
        for x in -map_half_size..(map_half_size + 1) {
            
            let tile_x = x as f32 * tile_size * tile_scale;
            let tile_y = y as f32 * tile_size * tile_scale;

            let tile = mapgen.get_tile(tile_x.into(), tile_y.into());

            let mut sprite = TextureAtlasSprite::new(tile.index);
            sprite.color = tile.color;

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    sprite: sprite,
                    transform: Transform {
                        translation: Vec3::new(tile_x, tile_y, 0.0),
                        scale: Vec3::splat(tile_scale),
                        ..Default::default()
                    },
                    ..default()
                },
            ));
        }
    }
}

fn respawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    keyboard: Res<Input<KeyCode>>,
    query: Query<Entity>,
) {
    if keyboard.just_pressed(KeyCode::Space) {

        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        spawn_map(commands, asset_server, texture_atlases);
    }
}