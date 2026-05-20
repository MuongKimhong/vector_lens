use bevy::prelude::*;
use bevy::tasks::{block_on, Task};
use bevy::tasks::futures_lite::future;
use crate::utils::create_log_with_timestamp;
use polars::prelude::{col, NamedFrom, Series, lit, DataFrameOps,
    CsvReadOptions, CsvWriter, DataFrame, SerReader, SerWriter, IntoLazy,
    datatypes::{Float64Type, Float32Type}, IndexOrder
};
use ndarray::ArrayBase;
use linfa::prelude::*;

use super::*;

// cubic spline rendering example https://bevy.org/examples/math/cubic-splines/

pub const OPERATOR_SIZE: Vec2 = Vec2::new(55.0, 55.0);

#[derive(Default, Debug, Clone)]
pub enum OpConnectButtonType {
    #[default]
    None,
    Input,
    Output
}

#[derive(Component, Clone)]
pub struct OpBox {
    pub id: Uuid,
    pub name: String,
}

impl OpBox {
    pub fn new(id: Uuid, name: &str) -> Self {
        Self {
            id,
            name: name.to_string()
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct OpConnectButton {
    pub connected: bool,
    pub button_type: OpConnectButtonType
}

impl OpConnectButton {
    pub fn new_as_input() -> Self {
        Self {
            connected: false,
            button_type: OpConnectButtonType::Input
        }
    }

    pub fn new_as_output() -> Self {
        Self {
            connected: false,
            button_type: OpConnectButtonType::Output
        }
    }
}

#[derive(Component)]
pub struct OpConnectionLine {
    pub input_button_entity: Option<Entity>,
    pub output_button_entity: Option<Entity>
}

impl OpConnectionLine {
    pub fn new(input_entity: Option<Entity>, output_entity: Option<Entity>) -> Self {
        Self {
            input_button_entity: input_entity,
            output_button_entity: output_entity
        }
    }
}

pub fn handle_op_background_execution_system(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut processing_tasks: Query<&mut ProcessingTask>,
    mut operator_q: Query<(Entity, &OperatorNameEntity, &mut Operator)>,
    mut text_colors: Query<&mut TextColor>,
    mut console_log: ResMut<ConsoleLog>,
    sender: Option<Res<TaskChannelSender>>
) {
    let Some(executing_op_entity) = ui_state.executing_operator else {
        return;
    };

    let mut current_op_result = DataValue::None;
    let mut next_op_entity: Option<Entity> = None;

    {
        let Ok((entity, op_name_entity, op)) = operator_q.get_mut(executing_op_entity) else {
            return;
        };

        // currently, this is a task being executed
        if let Ok(mut task) = processing_tasks.get_mut(executing_op_entity) {
            let task_result = block_on(future::poll_once(&mut task.0));

            if let Some(result) = task_result {
                commands.entity(executing_op_entity).remove::<ProcessingTask>();

                if let Ok(mut color) = text_colors.get_mut(op_name_entity.0) {
                    *color = TextColor::default();
                }

                // current operator produce error result, stop the process
                if result == DataValue::Error {
                    ui_state.is_running = false;
                    ui_state.executing_operator = None;
                    return;
                }
                current_op_result = result;
                next_op_entity = op.next_operator;

                // point executing_operator to next op
                ui_state.executing_operator = op.next_operator;

                if op.next_operator.is_none() {
                    ui_state.is_running = false;

                    let log = create_log_with_timestamp("Finished process");
                    console_log.new_message(LogType::Success(log));
                    return;
                }
            }
        }

        // no background task, create one
        else if let Some(sender) = sender {
            let task = op.spawn_task(&sender.0);
            commands.entity(entity).insert(ProcessingTask(task));

            if let Ok(mut color) = text_colors.get_mut(op_name_entity.0) {
                color.0 = Color::srgb(1.0, 1.0, 0.0);
            }
        }
    }

    {
        if next_op_entity.is_some() && current_op_result != DataValue::None {
            let Ok((_, _, mut operator)) = operator_q.get_mut(next_op_entity.unwrap()) else {
                return;
            };

            operator.input = current_op_result;
        }
    }
}

pub fn handle_insert_task_channel_resource_system(
    mut commands: Commands,
    sender: Option<Res<TaskChannelSender>>,
    receiver: Option<Res<TaskChannelReceiver>>
) {
    if sender.is_none() && receiver.is_none() {
        let (sender, receiver) = unbounded::<TaskChannelEvent>();
        commands.insert_resource(TaskChannelSender(sender));
        commands.insert_resource(TaskChannelReceiver(receiver));
    }
}

pub fn listen_to_task_channel_receiver_system(
    receiver: Option<Res<TaskChannelReceiver>>,
    mut console_log: ResMut<ConsoleLog>,
    mut test_set: ResMut<TestSet>,
    mut pre_read_content: ResMut<PreReadCsvOrExcelContent>,
    sender: Option<Res<TaskChannelSender>>
) {
    let Some(receiver) = receiver else {
        return;
    };

    while let Ok(channel_event) = receiver.0.try_recv() {
        match channel_event {
            TaskChannelEvent::LogMessage(log_type) => console_log.new_message(log_type),
            TaskChannelEvent::SetTestData(data) => {
                match data {
                    DataValue::Table(_) => test_set.0 = Some(data.clone()),
                    _ => {}
                }
            }
            TaskChannelEvent::LinearRegressionPrediction(data) => {
                println!("prediction data {:?}", data);

                // let Some(sender) = sender else {
                //     return;
                // };

                let test_df = test_set.0.as_mut().unwrap();

                match test_df {
                    DataValue::Table(test_df) => {
                        let num_test_rows = test_df.height();
                        let num_features = data.feature_names.len();

                        println!("[LR] Preparing test features for prediction...");

                        // Extract the test features using the exact same columns used during training
                        let test_x_df = match test_df.select(&data.feature_names) {
                            Ok(sub_df) => sub_df,
                            Err(e) => {
                                println!("test_x_df error {:?}", e);
                                return;
                            }
                        };

                        let test_x_flat: Vec<f32> = match test_x_df.to_ndarray::<Float32Type>(IndexOrder::C) {
                            Ok(matrix) => matrix.into_raw_vec(),
                            Err(e) => {
                                println!("test x flat error {:?}", e);
                                return;
                            }
                        };

                        // Reconstruct the 2D matrix structure for Linfa
                        let test_records: ndarray::Array2<f32> = ArrayBase::from_shape_vec((num_test_rows, num_features), test_x_flat).unwrap();

                        println!("[LR] Running predictions...");
                        // model.predict() outputs a 1D Array containing your predicted numeric values
                        let predictions = data.model.predict(&test_records);

                        let y_df = match test_df.select(&[&data.target_name as &str]) {
                            Ok(sub_df) => sub_df,
                            Err(e) => {
                                println!("[LR Error] Target column not found in test data: {:?}", e);
                                return;
                            }
                        };

                        // 2. Flatten the true target data into a Vec<f32>
                        let y_flat_vec: Vec<f32> = match y_df.to_ndarray::<Float32Type>(IndexOrder::C) {
                            Ok(matrix) => matrix.into_raw_vec(),
                            Err(e) => {
                                println!("[LR Error] Failed to flatten true targets: {:?}", e);
                                return;
                            }
                        };

                        // 3. Reconstruct as a Linfa-compatible target array
                        let ground_truth_targets: ndarray::Array1<f32> = ArrayBase::from_shape_vec(num_test_rows, y_flat_vec).unwrap();

                        println!("[LR] Evaluating model performance metrics...");

                        // 4. Pass the ground truth targets matrix into the evaluation functions
                        let r2_score = match predictions.r2(&ground_truth_targets) {
                            Ok(score) => score,
                            Err(e) => {
                                println!("[LR Warning] R2 calculation failed: {:?}", e);
                                0.0
                            }
                        };

                        let mse_score = match predictions.mean_squared_error(&ground_truth_targets) {
                            Ok(mse) => mse,
                            Err(e) => {
                                println!("[LR Warning] MSE calculation failed: {:?}", e);
                                0.0
                            }
                        };

                        // Calculate Root Mean Squared Error (RMSE)
                        let rmse_score = mse_score.sqrt();

                        println!("[LR] Evaluation - R² Accuracy Score: {:.4} ({:.1}% variance explained)", r2_score, r2_score * 100.0);
                        println!("[LR] Evaluation - Mean Squared Error (MSE): {:.4}", mse_score);
                        println!("[LR] Evaluation - Avg Error Distance (RMSE): {:.4}", rmse_score);

                        println!("[LR DEBUG] Performance stats calculated -> R2: {}, RMSE: {}", r2_score, rmse_score);
                    },
                    _ => {}
                };
            }
            // TaskChannelEvent::UpdatePreReadContentAfterSelectAttributes(data) => {
            //     match data {
            //         DataValue::Table(_) => pre_read_content.0 = data.clone(),
            //         _ => {}
            //     }
            // }
            _ => {}
        }
    }
}
