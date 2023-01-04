use bevy::prelude::*;
use std::ops::Add;
use std::ops::Mul;
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
const YELLOW_COLOR: Color = Color::rgb(240.0/255.0, 240.0/255.0, 127.0/255.0);
const BLUE_COLOR: Color   = Color::rgb(0.0/255.0, 255.0/255.0, 213.0/255.0);
const GREEN_COLOR: Color  = Color::rgb(0.0/255.0, 255.0/255.0, 17.0/255.0);

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
    
    PolarDesert,

    SubpolarDryTundra,
    SubpolarMoistTundra,
    SubpolarWetTundra,
    SubpolarRainTundra,
    
    BorealDesert,
    BorealDryScrub,
    BorealMoistForest,
    BorealWetForest,
    BorealRainForest,

    TemperateDesert,
    TemperateDesertScrub,
    TemperateSteppe,
    TemperateMoistForest,
    TemperateWetForest,
    TemperateRainForest,

    SubtropicalDesert,
    SubtropicalDesertScrub,
    SubtropicalThornWoodland,
    SubtropicalDryForest,
    SubtropicalMoistForest,
    SubtropicalWetForest,
    SubtropicalRainForest,
    
    TropicalDesert,
    TropicalDesertScrub,
    TropicalThornWoodland,
    TropicalVeryDryForest,
    TropicalDryForest,
    TropicalMoistForest,
    TropicalWetForest,
    TropicalRainForest,
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

        if temperature <= 0.0 { // polar
           return Biome::PolarDesert;
        } else if temperature > 0.0 && temperature <= 3.0 { // subpolar
            if percipitation <= 125.0 {
                return Biome::SubpolarDryTundra;
            } else if percipitation > 125.0 && percipitation <= 250.0 {
                return Biome::SubpolarMoistTundra;
            } else if percipitation > 250.0 && percipitation <= 500.0 {
                return Biome::SubpolarWetTundra;
            } else {
                return Biome::SubpolarRainTundra;
            }
        } else if temperature > 3.0 && temperature <= 6.0 { // boreal
            if percipitation <= 125.0 {
                return Biome::BorealDesert;
            } else if percipitation > 125.0 && percipitation <= 250.0 {
                return Biome::BorealDryScrub;
            } else if percipitation > 250.0 && percipitation <= 500.0 {
                return Biome::BorealMoistForest;
            } else if percipitation > 500.0 && percipitation <= 1000.0 {
                return Biome::BorealWetForest;
            } else {
                return Biome::BorealRainForest;
            }
        } else if temperature > 6.0 && temperature <= 12.0 { // temperate
            if percipitation <= 125.0 {
                return Biome::TemperateDesert;
            } else if percipitation > 125.0 && percipitation <= 250.0 {
                return Biome::TemperateDesertScrub;
            } else if percipitation > 250.0 && percipitation <= 500.0 {
                return Biome::TemperateSteppe;
            } else if percipitation > 500.0 && percipitation <= 1000.0 {
                return Biome::TemperateMoistForest;
            } else if percipitation > 1000.0 && percipitation <= 2000.0 {
                return Biome::TemperateWetForest;
            } else {
                return Biome::TemperateRainForest;
            }
        } else if temperature > 12.0 && temperature <= 24.0 { // subtropical
            if percipitation <= 125.0 {
                return Biome::SubtropicalDesert;
            } else if percipitation > 125.0 && percipitation <= 250.0 {
                return Biome::SubtropicalDesertScrub;
            } else if percipitation > 250.0 && percipitation <= 500.0 {
                return Biome::SubtropicalThornWoodland;
            } else if percipitation > 500.0 && percipitation <= 1000.0 {
                return Biome::SubtropicalDryForest;
            } else if percipitation > 1000.0 && percipitation <= 2000.0 {
                return Biome::SubtropicalMoistForest;
            } else if percipitation > 2000.0 && percipitation <= 4000.0 {
                return Biome::SubtropicalWetForest;
            } else {
                return Biome::SubtropicalRainForest;
            }
        } else { // tropical
            if percipitation <= 125.0 {
                return Biome::TropicalDesert;
            } else if percipitation > 125.0 && percipitation <= 250.0 {
                return Biome::TropicalDesertScrub;
            } else if percipitation > 250.0 && percipitation <= 500.0 {
                return Biome::TropicalThornWoodland;
            } else if percipitation > 500.0 && percipitation <= 1000.0 {
                return Biome::TropicalVeryDryForest;
            } else if percipitation > 1000.0 && percipitation <= 2000.0 {
                return Biome::TropicalDryForest;
            } else if percipitation > 2000.0 && percipitation <= 4000.0 {
                return Biome::TropicalMoistForest;
            } else if percipitation > 4000.0 && percipitation <= 8000.0 {
                return Biome::TropicalWetForest;
            } else {
                return Biome::TropicalRainForest;
            }
        }

    }

    fn pick_tile(&self, biome: Biome, temperature: f64, percipitation: f64) -> Tile {
        match biome {
            Biome::Ocean => Biome::ocean_tile(temperature, percipitation),

            Biome::PolarDesert => Biome::polar_desert_tile(temperature, percipitation),

            Biome::SubpolarDryTundra => Biome::subpolar_dry_tundra_tile(temperature, percipitation),
            Biome::SubpolarMoistTundra => Biome::subpolar_moist_tundra_tile(temperature, percipitation),
            Biome::SubpolarWetTundra => Biome::subpolar_wet_tundra_tile(temperature, percipitation),
            Biome::SubpolarRainTundra => Biome::subpolar_rain_tundra_tile(temperature, percipitation),
            
            Biome::BorealDesert => Biome::boreal_desert_tile(temperature, percipitation),
            Biome::BorealDryScrub => Biome::boreal_dry_scrub_tile(temperature, percipitation),
            Biome::BorealMoistForest => Biome::boreal_moist_forest_tile(temperature, percipitation),
            Biome::BorealWetForest => Biome::boreal_wet_forest_tile(temperature, percipitation),
            Biome::BorealRainForest => Biome::boreal_rain_forest_tile(temperature, percipitation),
        
            Biome::TemperateDesert => Biome::temperate_desert_tile(temperature, percipitation),
            Biome::TemperateDesertScrub => Biome::temperate_desert_scrub_tile(temperature, percipitation),
            Biome::TemperateSteppe => Biome::temperate_steppe_tile(temperature, percipitation),
            Biome::TemperateMoistForest => Biome::temperate_moist_forest_tile(temperature, percipitation),
            Biome::TemperateWetForest => Biome::temperate_wet_forest_tile(temperature, percipitation),
            Biome::TemperateRainForest => Biome::temperate_rain_forest_tile(temperature, percipitation),
        
            Biome::SubtropicalDesert => Biome::subtropical_desert_tile(temperature, percipitation),
            Biome::SubtropicalDesertScrub => Biome::subtropical_desert_scrub_tile(temperature, percipitation),
            Biome::SubtropicalThornWoodland => Biome::subtropical_thorn_woodland_tile(temperature, percipitation),
            Biome::SubtropicalDryForest => Biome::subtropical_dry_forest_tile(temperature, percipitation),
            Biome::SubtropicalMoistForest => Biome::subtropical_moist_forest_tile(temperature, percipitation),
            Biome::SubtropicalWetForest => Biome::subtropical_wet_forest_tile(temperature, percipitation),
            Biome::SubtropicalRainForest => Biome::subtropical_rain_forest_tile(temperature, percipitation),
            
            Biome::TropicalDesert => Biome::tropical_desert_tile(temperature, percipitation),
            Biome::TropicalDesertScrub => Biome::tropical_desert_scrub_tile(temperature, percipitation),
            Biome::TropicalThornWoodland => Biome::tropical_thorn_woodland_tile(temperature, percipitation),
            Biome::TropicalVeryDryForest => Biome::tropical_very_dry_forest_tile(temperature, percipitation),
            Biome::TropicalDryForest => Biome::tropical_dry_forest_tile(temperature, percipitation),
            Biome::TropicalMoistForest => Biome::tropical_moist_forest_tile(temperature, percipitation),
            Biome::TropicalWetForest => Biome::tropical_wet_forest_tile(temperature, percipitation),
            Biome::TropicalRainForest => Biome::tropical_rain_forest_tile(temperature, percipitation),
        }
    }

    fn get_heights(&self, r_dis: f64, x: f64, y: f64) -> (f64, f64) {
        let globe_noise = self.height_noise.get_value(x, y) * (1.0 - (r_dis + 0.3 + 0.4 * self.height_noise.get_value(-x, -y)));
        let height = 9000.0 * globe_noise - 1000.0;

        let mut absl_height = height;
        if absl_height < 0.0 {
            absl_height = 0.0;
        } 

        (height, absl_height)
    }

    fn get_partial_temp(&self, absl_height: f64, y_dis: f64, lapse_rate: f64, x: f64, y: f64) -> f64 {
        let noisy_temp = 20.0 * self.temperature_noise.get_value(x, y) + 5.0;
        let temperature = -40.0 * y_dis + noisy_temp - (lapse_rate * absl_height);

        temperature
    }

    fn get_percip_temp(&self, absl_height: f64, y_dis: f64, partial_temp: f64, x: f64, y: f64) -> (f64, f64) {
        
        let water_dist = 1.0 - (5.5 * y_dis.abs());
        
        let mut temp_clamp = partial_temp;
        if temp_clamp < 0.0 {
            temp_clamp = 0.0;
        }
        if temp_clamp > 40.0 {
            temp_clamp = 0.0;
        }

        let mut evap_prob = 1.0 - ((temp_clamp - 20.0) / 20.0).abs();
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

            if true_temp > 25.0 && true_temp < 35.0 {
                if absl_height < 3500.0 {
                    water_map +=  (1.0 - (absl_height / 3500.0));
                }
    
                if water_map > 0.99 {
                    water_map = 0.99;
                }
            }
        }

        let humidity = (0.40 * water_map) + (0.30 * water_dist) + (0.30 * self.humidity_noise.get_value(x, y));

        let mut percipitation = 16000.0 * humidity;
        let percipitation_cap = 500.0 * true_temp - 80.0;
        if percipitation > percipitation_cap {
            percipitation = percipitation_cap;
        }

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
        let mut humidity_clamp = percipitation as f32;

        let height_color = Color::rgb(height_clamp, height_clamp, height_clamp);
        let temperature_color = Color::rgb(temp_clamp, temp_clamp, temp_clamp);
        let humidity_color = Color::rgb(humidity_clamp, humidity_clamp, humidity_clamp);

        let mix_color  = Color::rgb(1.0 * temp_clamp, 1.0 * height_clamp, 0.0 * humidity_clamp);

        let biome: Biome = self.pick_biome(height, temperature, percipitation);

        let tile = self.pick_tile(biome, temperature, percipitation);

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

    fn calculate_grass_color(temperature: f64, percipitation: f64) -> Color {

        let percip_p = (percipitation + 10000.0) / 10000.0;
        let temp_p = temperature / 30.0;

        let mut deadness = ((1.0 - percip_p) + temp_p) / 2.0;
        if deadness < 0.0 {
            deadness = 0.0;
        }

        let mut alpine_p = 0.6 * (1.0 - temp_p) + 0.4 * (deadness);
        if alpine_p < 0.0 {
            alpine_p = 0.0;
        }
        
        let green_color = GREEN_COLOR.mul(deadness as f32);
        let yellow_color = YELLOW_COLOR.mul(percip_p as f32);
        let blue_color = BLUE_COLOR.mul(alpine_p as f32);

        let grass_color = green_color.add(yellow_color).mul(0.5);
        
        grass_color
    }

    fn ocean_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 3,
            color: Color::rgb(0.0, 0.2, 0.8),
        }
    }

    fn polar_desert_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 3,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }

    // Subpolar ------------------

    fn subpolar_dry_tundra_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 4,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }

    fn subpolar_moist_tundra_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subpolar_wet_tundra_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subpolar_rain_tundra_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    // Boreal ------------------

    fn boreal_desert_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn boreal_dry_scrub_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn boreal_moist_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn boreal_wet_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn boreal_rain_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    // Temperate ------------------

    fn temperate_desert_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn temperate_desert_scrub_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn temperate_steppe_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn temperate_moist_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn temperate_wet_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn temperate_rain_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    // Subtropical ------------------

    fn subtropical_desert_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subtropical_desert_scrub_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }
    
    fn subtropical_thorn_woodland_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subtropical_dry_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subtropical_moist_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subtropical_wet_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn subtropical_rain_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

     // Tropical ------------------

    fn tropical_desert_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 2,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }

    fn tropical_desert_scrub_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 1,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }

    fn tropical_thorn_woodland_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn tropical_very_dry_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn tropical_dry_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn tropical_moist_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn tropical_wet_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
        }
    }

    fn tropical_rain_forest_tile(temperature: f64, percipitation: f64) -> Tile {
        Tile {
            index: 0,
            color: Biome::calculate_grass_color(temperature, percipitation),
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