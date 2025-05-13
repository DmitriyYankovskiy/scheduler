use tauri_plugin_dialog::DialogExt;
use {
    scheduler::{
        models::{csv, ScheduleModel},
        Schedule,
    },
    tauri::AppHandle,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn work_with(app: AppHandle, file: &str, aging: usize, shuffling: bool, greedily: bool) -> u64 {
    let file = file.to_string();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file.as_bytes());
    // let mut schedule: Schedule = serde_json::from_str::<ScheduleModel>(&file).unwrap().into();
    let mut schedule = Schedule::new(ScheduleModel::deserialize_csv(&mut reader).unwrap().into());

    let time = std::time::Instant::now();

    schedule.optimize(0.999, aging, shuffling, greedily, || ());

    let dur = time.elapsed();
    let cost = schedule.cost;
    println!("results cost: {}", cost);
    println!("calculation time: {}", dur.as_secs_f32());

    app.dialog()
        .file()
        .add_filter("Schdedule table", &["csv", "txt"])
        .save_file(|path| {
            let path = path.unwrap();
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(&path.as_path().unwrap())
                .unwrap();
            ScheduleModel::from(schedule)
                .serialize_csv(&mut writer)
                .unwrap();
        });

    cost
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init()) // Add this line
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![work_with])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
