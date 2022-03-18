use crate::{spawn_spheres::OriginalPosition, parsing_function::FormulaParser};

use super::spawn_spheres::Sphere;
use bevy::prelude::*;

use bevy_egui::{egui::{self}, EguiContext, EguiPlugin};

use std::sync::{Arc};

pub struct MoveSpheres;

impl Plugin for MoveSpheres {
    fn build(&self, app: &mut App) {

        let parser = FormulaParser::new();

        app
            .add_plugin(EguiPlugin)
            .add_event::<ResetEvent>()
            .insert_resource(parser)
            .add_startup_system(setup_ui_state)
            .add_system(ui_setup.label("ui_update"))
            .add_system(move_spheres.after("ui_update"));
    }
}

fn setup_ui_state(
    mut commands: Commands,
    parser: Res<FormulaParser>,
) {
    commands.insert_resource(UiState {
            x_func: Arc::clone(&parser.parse("x")),
            // Cool Y formula: sin(x / time * 25) * 25
            y_func: Arc::clone(&parser.parse("sin(x - time) * 10")),
            z_func: Arc::clone(&parser.parse("z")),
            x_string: String::from("x"),
            y_string: String::from("sin(x - time) * 10"),
            z_string: String::from("z"),
            error: "".to_string()
        });
}

struct UiState {
    x_func: Arc<dyn Fn(f32, Vec3) -> Result<f32, String> + Send + Sync>,
    y_func: Arc<dyn Fn(f32, Vec3) -> Result<f32, String> + Send + Sync>,
    z_func: Arc<dyn Fn(f32, Vec3) -> Result<f32, String> + Send + Sync>,
    x_string: String,
    y_string: String,
    z_string: String,
    error: String,
}

struct ResetEvent;

fn ui_setup(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut ev_reset: EventWriter<ResetEvent>,
    parser: Res<FormulaParser>,
) {
    egui::Window::new("Simulation Options").show(egui_context.ctx_mut(), |ui| {

        let mut style: egui::Style = (*ui.ctx().style()).clone();
        style.override_text_style = Some(egui::TextStyle::Heading);
        ui.ctx().set_style(style);

        ui.vertical(|ui| {
            ui.label("X Graph Function: ");
            if ui.text_edit_singleline(&mut ui_state.x_string).changed() {
                ui_state.x_func = Arc::clone(&parser.parse(&ui_state.x_string))
            }
            ui.label("Y Graph Function: ");
            if ui.text_edit_singleline(&mut ui_state.y_string).changed() {
                ui_state.y_func = Arc::clone(&parser.parse(&ui_state.y_string))
            }
            ui.label("Z Graph Function: ");
            if ui.text_edit_singleline(&mut ui_state.z_string).changed() {
                ui_state.z_func = Arc::clone(&parser.parse(&ui_state.z_string))
            }
        });

        ui.vertical(|ui| {
            ui.label("Variables:");
            ui.label("  time: Time passed since simulation start.");
            ui.label("  x, y, and z: The axes of the current graph point.");

            ui.label("Functions:");
            ui.label("  sin() cos() tan()");

            ui.label("Binary Operations:");
            ui.label("  + - * /");

            // ui.label("Unary Operations:");
            // ui.label("  -");

            if !ui_state.error.is_empty() {
                ui.label("Function Error:");
                ui.label(&ui_state.error);
            }
            
            if ui.button("Reset").clicked() {
                *ui_state = UiState {
                    x_func: Arc::clone(&parser.parse("x")),
                    // Cool Y formula: sin(x / time * 25) * 25
                    y_func: Arc::clone(&parser.parse("y")),
                    z_func: Arc::clone(&parser.parse("z")),
                    x_string: String::from("x"),
                    y_string: String::from("y"),
                    z_string: String::from("z"),
                    error: "".to_string()
                };

                ev_reset.send(ResetEvent);
            }
        });
    });
}

fn move_spheres(
    mut spheres: Query<(&mut Transform, &OriginalPosition), With<Sphere>>,
    time: ResMut<Time>,
    mut ui_state: ResMut<UiState>,
    mut ev_reset: EventReader<ResetEvent>,
) {
    ui_state.error = String::new();
    let reset_event = ev_reset.iter().next();
    for (mut transform, original_transform) in spheres.iter_mut() {
        if reset_event.is_some() {
            *transform = original_transform.0;
        } else {
            match (ui_state.x_func)(time.seconds_since_startup() as f32, transform.translation) {
                Ok(output) => {
                    transform.translation.x = output;
                },
                Err(e) => {
                    ui_state.error = e;
                },
            }
            match (ui_state.y_func)(time.seconds_since_startup() as f32, transform.translation) {
                Ok(output) => {
                    transform.translation.y = output;
                },
                Err(e) => {
                    ui_state.error = e;
                },
            }
            match (ui_state.z_func)(time.seconds_since_startup() as f32, transform.translation) {
                Ok(output) => {
                    transform.translation.z = output;
                },
                Err(e) => {
                    ui_state.error = e;
                },
            }
        }
    }    
}
