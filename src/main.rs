use {
    chrono::prelude::{Local, Timelike},
    openweathermap::blocking::weather,
    std::{env, fs, process},
    subprocess::Exec,
    substring::Substring,
    unicode_segmentation::UnicodeSegmentation,
};

fn parse_args() {
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        match args[i].as_ref() {
            "-h" | "--help" => {
                println!("Usage:");
                println!("\t{} [OPTION]", args[0]);
                println!();
                println!("Help Options:");
                println!("\t-h, --help\t\tShow this message");
                println!("\t-c, --config\t\tSpecify a path to a config file");
                process::exit(0);
            }
            _ => (),
        }
    }
}

fn read_config() -> serde_json::Value {
    let args: Vec<String> = env::args().collect();
    let mut path = format!("{}/.config/hello-rs/config.json", env::var("HOME").unwrap());
    for i in 0..args.len() {
        match args[i].as_ref() {
            "-c" | "--config" => {
                path = args[i + 1].as_str().parse().unwrap();
            }
            _ => (),
        }
    }
    let file = fs::File::open(path).expect("Failed to open config file.");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("Failed to parse config file as a JSON.");
    json
}

fn get_release() -> String {
    let rel = Exec::cmd("lsb_release").arg("-sd")
        .capture()
        .unwrap()
        .stdout_str();
    if rel.len() > 32 {
        format!("{}...", rel.trim_matches('\"').substring(0, 28))
    } else {
        rel.trim_matches('\"').trim_end_matches('\n').trim_end_matches('\"').to_string()
    }
}

fn get_kernel() -> String {
    let uname = Exec::cmd("uname").arg("-sr")
        .capture()
        .unwrap()
        .stdout_str();
    if uname.len() > 32 {
        format!("{}...", uname.substring(0, 28))
    } else {
        uname.trim_end_matches('\n').to_string()
    }
}

fn check_updates() -> i32 {
    let mut total_updates = 0;

    let json = read_config();

    if json["package_managers"] == serde_json::json![null] {
        return -1;
    }

    if json["package_managers"].is_array() {
        let pm = json["package_managers"].as_array().unwrap();
        (0..pm.len()).for_each(|i| match pm[i].to_string().trim_matches('\"') {
            "pacman" => {
                let update_count = { Exec::cmd("checkupdates") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apt" => {
                let update_count = {
                    Exec::cmd("apt").arg("list").arg("-u")
                        | Exec::cmd("tail").arg("-n").arg("+2")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "xbps" => {
                let update_count =
                    { Exec::cmd("xbps-install").arg("-Sun") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "portage" => {
                let update_count = {
                    Exec::cmd("eix")
                        .arg("-u")
                        .arg("--format")
                        .arg("'<installedversions:nameversion>'")
                        | Exec::cmd("tail").arg("-1")
                        | Exec::cmd("cut").arg("-d").arg(" ").arg("-f2")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                if update_count.trim_end_matches('\n') != "matches" {
                    total_updates += update_count
                        .trim_end_matches('\n')
                        .parse::<i32>()
                        .unwrap_or(1);
                }
            }
            "apk" => {
                let update_count =
                    { Exec::cmd("apk").arg("-u").arg("list") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "dnf" => {
                let update_count = {
                    Exec::cmd("dnf").arg("check-update")
                        | Exec::cmd("tail").arg("-n").arg("+3")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            _ => (),
        });
    } else {
        let pm = &json["package_managers"];
        match pm.to_string().trim_matches('\"') {
            "pacman" => {
                let update_count = { Exec::cmd("checkupdates") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates = update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apt" => {
                let update_count = {
                    Exec::cmd("apt").arg("list").arg("-u")
                        | Exec::cmd("tail").arg("-n").arg("+2")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates = update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "xbps" => {
                let update_count =
                    { Exec::cmd("xbps-install").arg("-Sun") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates = update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "portage" => {
                let update_count = {
                    Exec::cmd("eix")
                        .arg("-u")
                        .arg("--format")
                        .arg("<installedversions:nameversion>")
                        | Exec::cmd("tail").arg("-1")
                        | Exec::cmd("cut").arg("-d").arg(" ").arg("-f2")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                if update_count != "matches" {
                    total_updates = update_count
                        .trim_end_matches('\n')
                        .parse::<i32>()
                        .unwrap_or(1);
                }
            }
            "apk" => {
                let update_count =
                    { Exec::cmd("apk").arg("-u").arg("list") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates = update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "dnf" => {
                let update_count = {
                    Exec::cmd("dnf").arg("check-update")
                        | Exec::cmd("tail").arg("-n").arg("+3")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates = update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            _ => (),
        }
    }

    total_updates
}

fn get_package_count() -> i32 {
    let mut total_packages = 0;

    let json = read_config();

    if json["package_managers"] == serde_json::json![null] {
        return -1;
    }

    if json["package_managers"].is_array() {
        let pm = json["package_managers"].as_array().unwrap();
        (0..pm.len()).for_each(|i| match pm[i].to_string().trim_matches('\"') {
            "pacman" => {
                let package_count = { Exec::cmd("pacman").arg("-Q") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages += package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apt" => {
                let package_count = {
                    Exec::cmd("dpkg-query").arg("-l")
                        | Exec::cmd("grep").arg("ii")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages += package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "xbps" => {
                let package_count =
                    { Exec::cmd("xbps-query").arg("-l") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_packages += package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "portage" => {
                let package_count =
                    { Exec::cmd("eix-installed").arg("-a") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_packages += package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apk" => {
                let package_count = { Exec::cmd("apk").arg("info") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages += package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "dnf" => {
                let package_count = {
                    Exec::cmd("dnf").arg("list").arg("installed")
                        | Exec::cmd("tail").arg("-n").arg("+2")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages += package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            _ => (),
        });
    } else {
        let pm = &json["package_managers"];
        match pm[0].to_string().trim_matches('\"') {
            "pacman" => {
                let package_count = { Exec::cmd("pacman").arg("-Q") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages = package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apt" => {
                let package_count = {
                    Exec::cmd("dpkg-query").arg("-l")
                        | Exec::cmd("grep").arg("ii")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages = package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "xbps" => {
                let package_count =
                    { Exec::cmd("xbps-query").arg("-l") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_packages = package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "portage" => {
                let package_count =
                    { Exec::cmd("eix-installed").arg("-a") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_packages = package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apk" => {
                let package_count = { Exec::cmd("apk").arg("info") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages = package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "dnf" => {
                let package_count = {
                    Exec::cmd("dnf").arg("list").arg("installed")
                        | Exec::cmd("tail").arg("-n").arg("+2")
                        | Exec::cmd("wc").arg("-l")
                }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_packages = package_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            _ => (),
        }
    }

    total_packages
}

fn get_song() -> String {
    let json = read_config();
    if json["song"] == false {
        return "".to_string();
    }
    let song = process::Command::new("playerctl")
        .arg("metadata")
        .arg("-f")
        .arg("{{ artist }} - {{ title }}")
        .output()
        .unwrap();
    let songerr = String::from_utf8_lossy(&song.stderr);
    let songname = String::from_utf8_lossy(&song.stdout);
    if songerr != "No players found" {
        if songname.len() > 32 {
            format!("{}...", songname.substring(0, 28))
        } else {
            songname.trim_end_matches('\n').to_string()
        }
    } else {
        "".to_string()
    }
}

fn upper_first(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn calc_whitespace(text: String) -> String {
    let size = 36 - text.graphemes(true).count();
    let final_string = format!("{}{}", " ".repeat(size), "│");
    format!("{}{}", text, final_string)
}

fn calc_with_hostname(text: String) -> String {
    let size = 46 - text.graphemes(true).count();
    let final_string = format!("{}{}", "─".repeat(size), "╮");
    format!("{}{}", text, final_string)
}

fn get_environment() -> String {
    env::var::<String>(ToString::to_string(&"XDG_CURRENT_DESKTOP")).unwrap_or_else(|_| "".to_string())
}

fn main() {
    parse_args();
    let json = read_config();
    let name = json
        .get("name")
        .expect("Couldn't find 'name' attribute.")
        .to_string();
    let location = json
        .get("location")
        .expect("Couldn't find 'location' attribute.")
        .to_string();
    let units = json
        .get("units")
        .expect("Couldn't find 'units' attribute.")
        .to_string();
    let lang = json
        .get("lang")
        .expect("Couldn't find 'lang' attribute.")
        .to_string();
    let api_key = json
        .get("api_key")
        .expect("Couldn't find 'api_key' attribute.")
        .to_string();
    let time_format = json
        .get("time_format")
        .expect("Couldn't find 'time_format' attribute.")
        .to_string();
    let dt = Local::now();
    let day = dt.format("%e").to_string();
    let date = match day.trim_start_matches(' ') {
        "1" | "21" | "31 " => format!("{} {}st", dt.format("%B"), day.trim_start_matches(' ')),
        "2" | "22" => format!("{} {}nd", dt.format("%B"), day.trim_start_matches(' ')),
        "3" | "23" => format!("{} {}rd", dt.format("%B"), day.trim_start_matches(' ')),
        _ => format!("{} {}th", dt.format("%B"), day.trim_start_matches(' ')),
    };
    let time = match time_format.trim_matches('\"') {
        "12h" => dt.format("%l:%M %p").to_string(),
        "24h" => dt.format("%H:%M").to_string(),
        _ => "off".to_string(),
    };
    let count = check_updates();
    let song = get_song();
    let packages = get_package_count();
    let hostname = json
        .get("hostname")
        .expect("Couldn't find 'hostname' attribute.")
        .to_string();
    let greeting = match dt.hour() {
        6..=11 => "🌇 Good morning",
        12..=17 => "🏙️ Good afternoon",
        18..=22 => "🌆 Good evening",
        _ => "🌃 Good night",
    };
    let mut time_icon = "";
    let deg;
    let icon_code;
    let icon;
    let main;
    let temp;

    if time != "off" {
        time_icon = match dt.hour() {
            0 | 12 => "🕛",
            1 | 13 => "🕐",
            2 | 14 => "🕑",
            3 | 15 => "🕒",
            4 | 16 => "🕓",
            5 | 17 => "🕔",
            6 | 18 => "🕕",
            7 | 19 => "🕖",
            8 | 20 => "🕗",
            9 | 21 => "🕘",
            10 | 22 => "🕙",
            11 | 23 => "🕚",
            _ => "🕛",
        };
    }

    match &weather(
        location.trim_matches('\"'),
        units.trim_matches('\"'),
        lang.trim_matches('\"'),
        api_key.trim_matches('\"'),
    ) {
        Ok(current) => {
            deg = if units.trim_matches('\"') == "imperial" {
                "F"
            } else {
                "C"
            };
            icon_code = &current.weather[0].icon;
            icon = match icon_code.as_ref() {
                "01d" => "☀️",
                "01n" => "🌙",
                "02d" => "⛅️",
                "02n" => "🌙",
                "03d" => "☁️",
                "03n" => "☁️",
                "04d" => "☁️",
                "04n" => "☁️",
                "09d" => "🌧️",
                "09n" => "🌧️",
                "10d" => "🌧️",
                "10n" => "🌧️",
                "11d" => "⛈️",
                "11n" => "⛈️",
                "13d" => "🌨️",
                "13n" => "🌨️",
                "40d" => "🌫️",
                "40n" => "🌫️",
                "50d" => "🌫️",
                "50n" => "🌫️",
                _ => "❓",
            };
            main = current.weather[0].main.to_string();
            temp = current.main.temp.to_string();
        }
        Err(e) => panic!("Could not fetch weather because: {}", e),
    }

    println!(
        "{}",
        calc_with_hostname(format!("╭─\x1b[32m{}\x1b[0m", hostname.trim_matches('\"')))
    );
    println!(
        "{}",
        calc_whitespace(format!("│ {}, {}!", greeting, name.trim_matches('\"')))
    );
    if time != "off" {
        println!(
            "{}",
            calc_whitespace(format!(
                "│ {} {}, {}",
                time_icon,
                date,
                time.trim_start_matches(' ')
            ))
        );
    }
    println!(
        "{}",
        calc_whitespace(format!(
            "│ {} {} {}°{}",
            icon,
            main,
            temp.substring(0, 2),
            deg
        ))
    );

    println!("{}", calc_whitespace(format!("│ 💻 {}", get_release())));
    println!("{}", calc_whitespace(format!("│ 🫀 {}", get_kernel())));
    match get_environment().as_ref() {
        "" => (),
        _ => println!(
            "{}",
            calc_whitespace(format!("│ 🖥️ {}", upper_first(get_environment())))
        ),
    }

    let update_count = count.to_string();

    let updates: String = match count {
        -1 => "none".to_string(),
        0 => "☑️ Up to date".to_string(),
        1 => "1️⃣ 1 update".to_string(),
        2 => "2️⃣ 2 updates".to_string(),
        3 => "3️⃣ 3 updates".to_string(),
        4 => "4️⃣ 4 updates".to_string(),
        5 => "5️⃣ 5 updates".to_string(),
        6 => "6️⃣ 6 updates".to_string(),
        7 => "7️⃣ 7 updates".to_string(),
        8 => "8️⃣ 8 updates".to_string(),
        9 => "9️⃣ 9 updates".to_string(),
        10 => "🔟 10 updates".to_string(),
        _ => format!("‼️ {} updates", update_count),
    };

    if updates != "none" {
        println!("{}", calc_whitespace(format!("│ {}", updates)));
    }

    match packages {
        -1 => (),
        0 => println!("{}", calc_whitespace("│ 📦 No packages".to_string())),
        1 => println!("{}", calc_whitespace("│ 📦 1 package".to_string())),
        _ => println!("{}", calc_whitespace(format!("│ 📦 {} packages", packages))),
    }

    match song.as_ref() {
        "" => (),
        _ => println!(
            "{}",
            calc_whitespace(format!("│ 🎵 {}", song.trim_matches('\n')))
        ),
    }

    println!("\u{2570}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{256f}");
}
