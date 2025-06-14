use std::sync::Arc;

use tauri::async_runtime::Mutex;
use tauri_plugin_dialog::DialogExt;
use {
    scheduler::{
        models::{csv, ScheduleModel},
        Schedule,
    },
    tauri::{AppHandle, Manager},
};

struct State {
    schedule: Arc<Mutex<Option<Schedule>>>,
    file: Arc<Mutex<Option<String>>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn optimize_schedule(
    app: AppHandle,
    aging: usize,
    shuffling: bool,
    greedily: bool,
) -> Result<u64, String> {
    // let mut schedule: Schedule = serde_json::from_str::<ScheduleModel>(&file).unwrap().into();

    let state = app.state::<State>();
    let mut schedule = state.schedule.lock().await;
    if let None = *schedule {
        let file = state.file.lock().await;
        let file = file.as_ref().ok_or("file was not selected")?;
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(String::as_bytes(&file));
        *schedule = Some(Schedule::new(
            ScheduleModel::deserialize_csv(&mut reader).unwrap().into(),
        ));
    };
    let time = std::time::Instant::now();
    let schedule = unsafe { schedule.as_mut().unwrap_unchecked() };
    schedule.optimize(0.999, aging, shuffling, greedily, || ());

    let cost = schedule.cost;
    let dur = time.elapsed();

    println!("results cost: {}", cost);
    println!("calculation time: {}", dur.as_secs_f32());

    Ok(cost)
}

#[tauri::command]
async fn download_file(app: AppHandle) -> Result<(), String> {
    let scheme = {
        let state = app.state::<State>();
        let schedule = state.schedule.lock().await;
        let schedule = if let Some(s) = &*schedule {
            s
        } else {
            return Err("File is not optimized".to_string());
        };
        schedule.scheme.clone()
    };
    app.dialog()
        .file()
        .add_filter("Schdedule table", &["csv", "txt"])
        .save_file(|path| {
            let path = path.unwrap();
            let mut writer = match csv::WriterBuilder::new().has_headers(false).from_path(
                if let Some(p) = &path.as_path() {
                    p
                } else {
                    return;
                },
            ) {
                Ok(w) => w,
                Err(_) => return,
            };

            ScheduleModel::from(scheme)
                .serialize_csv(&mut writer)
                .unwrap();
        });
    Ok(())
}

#[tauri::command]
async fn select_file(app: AppHandle, file: String) -> Result<(), ()> {
    let state = app.state::<State>();
    *state.file.lock().await = Some(file);
    *state.schedule.lock().await = None;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(State {
                schedule: Arc::new(Mutex::new(None)),
                file: Arc::new(Mutex::new(None)),
            });
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init()) // Add this line
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            optimize_schedule,
            download_file,
            select_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
