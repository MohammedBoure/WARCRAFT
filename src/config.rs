use astra_voxel_world::prelude::*;

use crate::state::*;

impl ViewerOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let tokens = args.into_iter().collect::<Vec<_>>();
        let mut options = Self::default();
        options.dev_world = tokens.iter().any(|arg| arg == "--dev-world");
        let mut args = tokens.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => options.help = true,
                "--dev-world" => {}
                "--seed" if options.dev_world => {
                    let value = args.next().ok_or("القيمة مفقودة بعد --seed")?;
                    options.seed = parse_u64(&value)?;
                }
                "--radius" if options.dev_world => {
                    let value = args.next().ok_or("القيمة مفقودة بعد --radius")?;
                    options.load_radius = value
                        .parse::<i64>()
                        .map_err(|error| format!("قيمة --radius غير صالحة: {error}"))?
                        .clamp(3, LOAD_RADIUS_MAX);
                }
                "--preset" if options.dev_world => {
                    let value = args.next().ok_or("القيمة مفقودة بعد --preset")?;
                    options.composition = VoxelWorldComposition::preset(&value)
                        .ok_or_else(|| format!("إعداد عالم غير معروف: {value}"))?;
                }
                unknown if unknown.starts_with('-') && !options.dev_world => {
                    return Err(format!(
                        "الخيار {unknown} خاص بالمطور. استخدم --dev-world أولاً"
                    ));
                }
                unknown => return Err(format!("خيار غير معروف: {unknown}")),
            }
        }

        Ok(options)
    }

    pub const fn help_text() -> &'static str {
        "نقطة الانهيار - لعبة عربية لموضوع Critical Point\n\nالاستخدام:\n  bevy_playground.exe [--help]\n  bevy_playground.exe --dev-world [خيارات]\n\nخيارات اللاعب:\n  --help              عرض هذه المساعدة والخروج\n\nخيارات المطور:\n  --dev-world         تفعيل خيارات اختبار العالم\n  --seed <u64>        بذرة العالم، عشرية أو 0xHEX\n  --radius <3..12>    نصف قطر تحميل المناطق\n  --preset <name>     balanced, lush, dry, frozen, volcanic, crystal, crater\n\nالتحكم:\n  WASD / الأسهم       الحركة\n  Shift               الركض\n  Space               القفز\n  زر الفأرة الأيسر    الحفر أو استخراج البلور\n  زر الفأرة الأيمن    وضع كتلة دعم\n  E                   التفاعل\n  Esc                 إيقاف مؤقت\n"
    }
}

fn parse_u64(value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).map_err(|error| format!("بذرة غير صالحة: {error}"))
    } else {
        trimmed
            .parse::<u64>()
            .map_err(|error| format!("بذرة غير صالحة: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_options_are_hidden_without_flag() {
        let error = ViewerOptions::parse(["--seed".to_string(), "12".to_string()])
            .expect_err("seed should require dev mode");
        assert!(error.contains("خاص بالمطور"));
    }

    #[test]
    fn dev_seed_accepts_hex() {
        let options = ViewerOptions::parse([
            "--dev-world".to_string(),
            "--seed".to_string(),
            "0x2A".to_string(),
        ])
        .unwrap();
        assert_eq!(options.seed, 42);
    }
}