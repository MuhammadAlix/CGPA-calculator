// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(mobile, tauri::mobile_entry_point)]


pub fn run() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![process_student_results, process_multiseme_csv_data])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}


use std::time::Duration;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Subject {
    pub semester_label: String,
    pub course_code: String,
    pub credit_hours: u8,
    pub obtained_marks: f32,
    pub total_marks: u8,
    pub grade: String,
}

#[derive(Debug, Serialize)]
pub struct SemesterResult {
    semester_label: String,
    gpa: f32,
}

#[derive(Debug, Serialize)]
pub struct FinalResult {
    semesters: Vec<SemesterResult>,
    cgpa: f32,
}

fn semester_type(label: &str) -> &str {
    if label.contains("Spring") {
        "Spring"
    } else {
        "Winter"
    }
}

fn grade_to_point(grade: &str) -> u8 {
    match grade {
        "A" => 5,
        "B" => 4,
        "C" => 3,
        "D" => 2,
        "F" => 1,
        _ => 0,
    }
}

fn parse_table_by_semester(table_text: &str) -> HashMap<String, Vec<Vec<String>>> {
    let mut semester_map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    let mut current_semester = String::new();

    for (i, line) in table_text.lines().enumerate() {
        let cols: Vec<String> = line.split('\t').map(|s| s.trim().to_string()).collect();
        if cols.len() < 5 { continue; }

        if !cols[1].is_empty() {
            current_semester = cols[1].clone();
        }

        if i == 0 || current_semester.is_empty() || cols[0] == "Sr" {
            continue;
        }

        semester_map.entry(current_semester.clone()).or_default().push(cols);
    }

    semester_map
}


fn convert_rows_to_subjects(semester_map: HashMap<String, Vec<Vec<String>>>) -> Vec<Subject> {
    let mut subjects = Vec::new();

    for (semester_label, rows) in semester_map {
        for row in rows {
            if row.len() < 12 { continue; }

            let course_code = row[3].clone();
            let credit_hours: u8 = row[5].split('(').next().unwrap_or("0").trim().parse().unwrap_or(0);
            let total = row[10].parse().unwrap_or(0.0);
            let grade = row[11].clone();

            let total_marks = match credit_hours {
                2 => 40,
                3 => 60,
                6 => 120,
                _ => 60,
            };

            subjects.push(Subject {
                semester_label: semester_label.clone(),
                course_code,
                credit_hours,
                obtained_marks: total,
                total_marks,
                grade,
            });
        }
    }

    subjects
}
// use headless_chrome::Browser;
// use headless_chrome::LaunchOptionsBuilder;
use headless_chrome::{Browser, LaunchOptionsBuilder};
fn fetch_result_tables(reg_number: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let browser = Browser::new(
        LaunchOptionsBuilder::default().headless(true).build().unwrap(),
    )?;
    let tab = browser.new_tab()?;

    tab.navigate_to("http://lms.uaf.edu.pk/login/index.php")?;
    tab.wait_until_navigated()?;
    tab.wait_for_element("#REG")?.click()?.type_into(reg_number)?;
    tab.wait_for_xpath("//input[@type='submit' and @value='Result']")?.click()?;
    std::thread::sleep(Duration::from_secs(5));

    let tables = tab.find_elements("table")?;
    Ok(tables.into_iter().map(|t| t.get_inner_text().unwrap_or_default()).collect())
}

fn calculate_gpa_cgpa(subjects: &[Subject], qpt_map: &HashMap<(u8, u8), f32>) -> FinalResult {
    let mut latest_subjects: HashMap<(String, String), &Subject> = HashMap::new();


    for subject in subjects {
        let key = (semester_type(&subject.semester_label).to_string(), subject.course_code.clone());
        match latest_subjects.get(&key) {
            Some(existing) => {
                if grade_to_point(&subject.grade) > grade_to_point(&existing.grade) {
                    latest_subjects.insert(key, subject);
                }
            }
            None => {
                latest_subjects.insert(key, subject);
            }
        }
    }

    let mut semester_map: HashMap<String, Vec<&Subject>> = HashMap::new();
    for subj in latest_subjects.values() {
        semester_map.entry(subj.semester_label.clone()).or_default().push(*subj);
    }

    fn parse_sem_order(label: &str) -> Option<(u32, u8)> {
        let parts: Vec<&str> = label.split_whitespace().collect();
        if parts.len() < 3 { return None; }

        let year = parts[2].split('-').next()?.parse::<u32>().ok()?;
        let sem_order = if parts[0] == "Winter" { 0 } else { 1 };
        Some((year, sem_order))
    }

    let mut sorted_semesters: Vec<_> = semester_map.iter().collect();
    sorted_semesters.sort_by_key(|(s, _)| parse_sem_order(s).unwrap_or((9999, 9)));

    let mut total_qp = 0.0;
    let mut total_ch = 0;
    let mut semesters_result = Vec::new();

    for (semester, subs) in sorted_semesters {
        let mut sem_qp = 0.0;
        let mut sem_ch = 0;

        for s in subs {
            let key = (s.total_marks, s.obtained_marks.round() as u8);
            if let Some(qp) = qpt_map.get(&key) {
                sem_qp += qp;
                sem_ch += s.credit_hours as u32;
            }else {
                eprintln!(
                     "❌ No QP found for course {} with marks {} out of {}",
                    s.course_code, s.obtained_marks, s.total_marks
                );
}
        }

        let gpa = if sem_ch == 0 { 0.0 } else { sem_qp / sem_ch as f32 };
        total_qp += sem_qp;
        total_ch += sem_ch;

        semesters_result.push(SemesterResult {
            semester_label: semester.clone(),
            gpa: (gpa * 100.0).round() / 100.0,
        });
    }

    let cgpa = if total_ch == 0 { 0.0 } else { total_qp / total_ch as f32 };

    FinalResult {
        semesters: semesters_result,
        cgpa: (cgpa * 100.0).round() / 100.0,
    }
}

#[tauri::command]
fn process_student_results(reg_number: String) -> Result<FinalResult, String> {
    let tables = fetch_result_tables(&reg_number).map_err(|e| e.to_string())?;
    let table = &tables[1];
    let parsed = parse_table_by_semester(table);
    let subjects = convert_rows_to_subjects(parsed);
    let qpt_map = build_qpt();
    Ok(calculate_gpa_cgpa(&subjects, &qpt_map))
}

#[tauri::command]
fn process_multiseme_csv_data(csvString: String) -> String {
    let mut semester_map: HashMap<String, Vec<Subj>> = HashMap::new();
    let qpt = build_qpt();

    for (i, line) in csvString.lines().skip(1).enumerate() {
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
            .push(Subj { ch, tm, om });
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
struct Subj {
    ch: u32,
    tm: u32,
    om: u32,
}

// #[derive(Debug, serde::Deserialize)]
// pub struct SemesterInput {
//     semesters: Vec<Vec<Vec<String>>>, // [semester][subject][ch, tm, om]
// }

use std::collections::HashMap;
fn build_qpt() -> HashMap<(u8, u8), f32> {
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
        (98, 24.0), (100, 24.0), (102, 24.0), (104, 24.0), (106, 24.0), (107, 24.0),
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

fn calculate_gpa(subjects: &[Subj], qpt: &HashMap<(u8, u8), f32>) -> (f32, f32, u32) {
    let mut total_points = 0.0;
    let mut total_credits = 0;

    for subject in subjects {
        if subject.ch == 0 || subject.tm == 0 {
            continue;
        }

        if let Some(&qp) = qpt.get(&(subject.tm as u8, subject.om as u8)) {
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


