use {
    scheduler::{
        models::{csv, ScheduleModel},
        Schedule,
    },
    std::{fs::File, io::Write, sync::{Arc, Mutex}}, tauri::AppHandle,
};
use tauri_plugin_dialog::DialogExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn work_with(app: AppHandle, file: &str) -> String {
    let file = file.to_string();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file.as_bytes());
    // let mut schedule: Schedule = serde_json::from_str::<ScheduleModel>(&file).unwrap().into();
    let mut schedule = Schedule::new(ScheduleModel::deserialize_csv(&mut reader).unwrap().into());

    let aging = 10000; //args.aging_opt.unwrap_or(scheduler::AGING_OPT_DEFAULT);

    let time = std::time::Instant::now();

    schedule.optimize(0.999, aging, true, true, || ());


    let dur = time.elapsed();
    println!("results cost: {}", schedule.cost);
    println!("calculation time: {}", dur.as_secs_f32());

    //let schedule = Arc::new(Mutex::new(schedule));

    app.dialog()
        .file()
        .add_filter("My Filter", &["png", "jpeg"])
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
    "Success".to_string()
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
