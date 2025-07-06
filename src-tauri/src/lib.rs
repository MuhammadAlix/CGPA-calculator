// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![process_multiseme_csv_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn process_multiseme_csv_data(csvString: String) -> String {
    let mut semester_map: HashMap<String, Vec<Subject>> = HashMap::new();
    let qpt = build_qpt();

    for (i, line) in csvString.lines().skip(1).enumerate() { // skip optional header
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            eprintln!("⚠️ Skipping invalid row {}: {:?}", i + 1, parts);
            continue;
        }

        let ch = parts[0].trim().parse::<u32>().unwrap_or(0);
        let tm = parts[1].trim().parse::<u32>().unwrap_or(0);
        let om = parts[2].trim().parse::<u32>().unwrap_or(0);
        let semester = parts[3].trim().to_string();

        if ch == 0 || tm == 0 {
            continue;
        }

        semester_map
            .entry(semester)
            .or_default()
            .push(Subject { ch, tm, om });
    }

    let mut total_qp = 0.0;
    let mut total_ch = 0;
    let mut output = String::new();

    for (semester, subjects) in &semester_map {
        let (gpa, qp, ch) = calculate_gpa(subjects, &qpt);
        total_qp += qp;
        total_ch += ch;
        output += &format!("{} GPA: {:.2}\n", semester, gpa);
    }

    let cgpa = if total_ch == 0 {
        0.0
    } else {
        total_qp / total_ch as f32
    };

    output += &format!("\n🎓 Cumulative CGPA (Normalized to 4.0): {:.2}", cgpa);

    output
}

#[derive(Debug)]
struct Subject {
    ch: u32,
    tm: u32,
    om: u32,
}

// #[derive(Debug, serde::Deserialize)]
// pub struct SemesterInput {
//     semesters: Vec<Vec<Vec<String>>>, // [semester][subject][ch, tm, om]
// }

use std::collections::HashMap;
fn build_qpt() -> HashMap<(u32, u32), f32> {
    let mut map = HashMap::new();

    let qps_60 = [
        (24, 3.0), (25, 3.5), (26, 4.0), (27, 4.5), (28, 5.0),
        (29, 5.5), (30, 6.0), (31, 6.33), (32, 6.67), (33, 7.0),
        (34, 7.33), (35, 7.67), (36, 8.0), (37, 8.33), (38, 8.67),
        (39, 9.0), (40, 9.33), (41, 9.67), (42, 10.0), (43, 10.33),
        (44, 10.67), (45, 11.0), (46, 11.33), (47, 11.67), (48, 12.0),
        (49, 12.0), (50, 12.0), (51, 12.0), (52, 12.0), (53, 12.0),
        (54, 12.0), (55, 12.0), (56, 12.0), (57, 12.0), (58, 12.0),
        (59, 12.0), (60, 12.0),
    ];

    let qps_40 = [
        (16, 2.0), (17, 2.5), (18, 3.0), (19, 3.5), (20, 4.0),
        (21, 4.33), (22, 4.67), (23, 5.0), (24, 5.33), (25, 5.67),
        (26, 6.0), (27, 6.33), (28, 6.67), (29, 7.0), (30, 7.33),
        (31, 7.67), (32, 8.0), (33, 8.0), (34, 8.0), (35, 8.0),
        (36, 8.0), (37, 8.0), (38, 8.0), (39, 8.0), (40, 8.0),
    ];

    let qps_120 = [
        (48, 6.0), (49, 6.5), (50, 7.0), (51, 7.5), (52, 8.0), (53, 8.5), (54, 9.0), (55, 9.5), (56, 10.0), (57, 10.5),
        (58, 11.0), (59, 11.5), (60, 12.0), (61, 12.5), (62, 13.0), (63, 13.17), (64, 13.34), (65, 13.67), (66, 14.0), (67, 14.5),
        (68, 15.0), (69, 15.17), (70, 15.34), (71, 15.67), (72, 16.0), (73, 16.5), (74, 17.0), (75, 17.17), (76, 17.34), (77, 17.67),
        (78, 18.0), (79, 18.5), (80, 19.0), (81, 19.17), (82, 19.34), (80, 19.67), (84, 20.0), (85, 20.5), (86, 21.0), (87, 21.17),
        (88, 21.34), (89, 21.67), (90, 22.0), (91, 21.5), (92, 23.0), (93, 23.17), (94, 23.34), (95, 23.67), (96, 24.0),
        (98, 24.0), (100, 24.0), (102, 24.0), (104, 24.0), (106, 24.0),
        (108, 24.0), (110, 24.0), (112, 24.0), (114, 24.0), (116, 24.0),
        (118, 24.0), (120, 24.0),
    ];

    for (om, qp) in qps_60 {
        map.insert((60, om), qp);
    }
    for (om, qp) in qps_40 {
        map.insert((40, om), qp);
    }
    for (om, qp) in qps_120 {
        map.insert((120, om), qp);
    }

    map
}

fn calculate_gpa(subjects: &[Subject], qpt: &HashMap<(u32, u32), f32>) -> (f32, f32, u32) {
    let mut total_points = 0.0;
    let mut total_credits = 0;

    for subject in subjects {
        if subject.ch == 0 || subject.tm == 0 {
            continue;
        }

        if let Some(&qp) = qpt.get(&(subject.tm, subject.om)) {
            let max_qp = match subject.tm {
                40 => 8.0,
                60 => 12.0,
                100 => 20.0,
                120 => 24.0,
                _ => qp, // Use raw if unknown
            };
            let normalized_qp = (qp / max_qp) * 4.0;
            total_points += normalized_qp * subject.ch as f32;
            total_credits += subject.ch;
        }
    }

    let gpa = if total_credits == 0 {
        0.0
    } else {
        total_points / total_credits as f32
    };

    (gpa, total_points, total_credits)
}


